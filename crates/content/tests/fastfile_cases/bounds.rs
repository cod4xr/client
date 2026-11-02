// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn every_truncated_prefix_and_short_capacity_preserves_complete_final_storage() {
    let input = stored(&inner(b"body", 4, 0, [0; 9]));
    let mut decoder = Fastfile5Decoder::new();
    let mut scratch = [0; 49];
    let mut output = [0xa5; 52];
    for end in 0..input.len() {
        assert!(
            decoder
                .decode(&input[..end], BUDGET, &mut scratch, &mut output)
                .is_err()
        );
        assert_eq!(output, [0xa5; 52]);
    }
    for length in 0..49 {
        assert_eq!(
            decoder
                .decode(&input, BUDGET, &mut scratch[..length], &mut output)
                .err(),
            Some(Fastfile5Error::ScratchCapacity)
        );
        assert_eq!(output, [0xa5; 52]);
    }
    for length in 0..48 {
        assert_eq!(
            decoder
                .decode(&input, BUDGET, &mut scratch, &mut output[..length])
                .err(),
            Some(Fastfile5Error::OutputCapacity)
        );
        assert_eq!(output, [0xa5; 52]);
    }
    decoder
        .decode(&input, BUDGET, &mut scratch, &mut output)
        .expect("exact bounds");
    assert_eq!(output[48..], [0xa5; 4]);
}

#[test]
fn each_host_budget_is_independent_with_exact_boundary_admission() {
    let input = stored(&inner(b"body", 4, 12, [1; 9]));
    let exact = Fastfile5Budget {
        max_file_bytes: input.len(),
        max_decoded_bytes: 48,
        max_total_block_bytes: 9,
        max_external_bytes: 12,
    };
    let mut decoder = Fastfile5Decoder::new();
    assert!(decoder.inspect(&input, exact).is_ok());
    for (budget, error) in [
        (
            Fastfile5Budget {
                max_file_bytes: input.len() - 1,
                ..exact
            },
            Fastfile5Error::FileBudget,
        ),
        (
            Fastfile5Budget {
                max_decoded_bytes: 47,
                ..exact
            },
            Fastfile5Error::DecodedBudget,
        ),
        (
            Fastfile5Budget {
                max_total_block_bytes: 8,
                ..exact
            },
            Fastfile5Error::BlockBudget,
        ),
        (
            Fastfile5Budget {
                max_external_bytes: 11,
                ..exact
            },
            Fastfile5Error::ExternalBudget,
        ),
    ] {
        assert_eq!(decoder.inspect(&input, budget), Err(error));
    }
}

#[test]
fn wide_block_totals_do_not_wrap_or_allocate_declared_memory() {
    let input = stored(&inner(&[], 0, u32::MAX, [u32::MAX; 9]));
    let budget = Fastfile5Budget {
        max_decoded_bytes: 44,
        max_total_block_bytes: u64::MAX,
        max_external_bytes: u64::MAX,
        ..BUDGET
    };
    let mut decoder = Fastfile5Decoder::new();
    let mut scratch = [0; 45];
    let mut output = [0; 44];
    let checked = decoder
        .decode(&input, budget, &mut scratch, &mut output)
        .expect("inert wide metadata");
    assert_eq!(
        checked.metadata().total_block_bytes(),
        u64::from(u32::MAX) * 9
    );
    assert_eq!(
        decoder.inspect(
            &input,
            Fastfile5Budget {
                max_total_block_bytes: u64::from(u32::MAX),
                ..budget
            }
        ),
        Err(Fastfile5Error::BlockBudget)
    );
    let huge_body = stored(&inner(&[], u32::MAX, 0, [0; 9]));
    assert_eq!(
        decoder.inspect(&huge_body, BUDGET),
        Err(Fastfile5Error::DecodedBudget)
    );
}

#[test]
fn signatures_versions_and_endianness_are_explicitly_selected() {
    let input = stored(&inner(&[], 0, 0, [0; 9]));
    let mut decoder = Fastfile5Decoder::new();
    for magic in [b"IWff0100", b"IWffu101", b"IWffu200"] {
        let mut bytes = input.clone();
        bytes[..8].copy_from_slice(magic);
        assert_eq!(
            decoder.inspect(&bytes, BUDGET),
            Err(Fastfile5Error::Profile)
        );
    }
    for version in [0_u32, 1, 4, 6, 0x0500_0000, u32::MAX] {
        let mut bytes = input.clone();
        bytes[8..12].copy_from_slice(&version.to_le_bytes());
        assert_eq!(
            decoder.inspect(&bytes, BUDGET),
            Err(Fastfile5Error::Profile)
        );
    }
}

#[test]
fn entropy_and_header_mutations_never_publish_an_incomplete_section() {
    let input = stored(&inner(b"body", 4, 0, [0; 9]));
    let mut decoder = Fastfile5Decoder::new();
    for index in 12..input.len() {
        for value in 0..=255 {
            let mut bytes = input.clone();
            bytes[index] = value;
            let mut scratch = [0; 49];
            let mut output = [0x9a; 52];
            match decoder.decode(&bytes, BUDGET, &mut scratch, &mut output) {
                Ok(checked) => assert_eq!(checked.contents(), b"body"),
                Err(_) => assert_eq!(output, [0x9a; 52]),
            }
            assert_eq!(output[48..], [0x9a; 4]);
        }
    }
}
