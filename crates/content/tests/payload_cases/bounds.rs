// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    BUDGET, HELLO, HELLO_CRC, ZipPayloadBudget, ZipPayloadDecoder, ZipPayloadError, entry, source,
};

#[test]
fn exact_work_scratch_and_final_capacity_bounds_preserve_output_on_failure() {
    let fixture = source(HELLO, 5, HELLO_CRC, 8);
    let exact = ZipPayloadBudget {
        max_compressed_bytes: 7,
        max_uncompressed_bytes: 5,
    };
    for (budget, scratch_len, output_len, expected) in [
        (
            ZipPayloadBudget {
                max_compressed_bytes: 6,
                ..exact
            },
            6,
            5,
            ZipPayloadError::InputBudget,
        ),
        (
            ZipPayloadBudget {
                max_uncompressed_bytes: 4,
                ..exact
            },
            6,
            5,
            ZipPayloadError::OutputBudget,
        ),
        (exact, 5, 5, ZipPayloadError::ScratchCapacity),
        (exact, 6, 4, ZipPayloadError::OutputCapacity),
    ] {
        let mut output = vec![0xa5; output_len];
        let before = output.clone();
        assert_eq!(
            ZipPayloadDecoder::new()
                .decode(
                    entry(&fixture.bytes),
                    budget,
                    &mut vec![0xff; scratch_len],
                    &mut output
                )
                .err(),
            Some(expected)
        );
        assert_eq!(output, before);
    }
    let mut output = [0; 5];
    let checked = ZipPayloadDecoder::new()
        .decode(entry(&fixture.bytes), exact, &mut [0; 6], &mut output)
        .expect("Exact capacities");
    assert_eq!(checked.bytes(), b"hello");
}

#[test]
fn false_output_lengths_and_checksums_reject_before_final_writes() {
    for length in [0, 1, 4, 6, 10] {
        let fixture = source(HELLO, length, HELLO_CRC, 8);
        let mut output = [0xa5; 11];
        assert_eq!(
            ZipPayloadDecoder::new()
                .decode(entry(&fixture.bytes), BUDGET, &mut [0xff; 12], &mut output)
                .err(),
            Some(ZipPayloadError::OutputLength)
        );
        assert_eq!(output, [0xa5; 11]);
    }
    for (packed, method) in [(&b"hello"[..], 0), (HELLO, 8)] {
        let fixture = source(packed, 5, HELLO_CRC ^ 1, method);
        let mut output = [0xa5; 5];
        assert_eq!(
            ZipPayloadDecoder::new()
                .decode(entry(&fixture.bytes), BUDGET, &mut [0xff; 6], &mut output)
                .err(),
            Some(ZipPayloadError::Checksum)
        );
        assert_eq!(output, [0xa5; 5]);
    }
}

#[test]
fn output_work_budget_rejects_large_declarations_before_needing_large_buffers() {
    let fixture = source(HELLO, 3_000_000_000, 0, 8);
    assert_eq!(
        ZipPayloadDecoder::new()
            .decode(entry(&fixture.bytes), BUDGET, &mut [], &mut [])
            .err(),
        Some(ZipPayloadError::OutputBudget)
    );
}
