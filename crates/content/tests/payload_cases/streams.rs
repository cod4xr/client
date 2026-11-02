// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    BUDGET, HELLO, HELLO_CRC, ZipPayloadDecoder, ZipPayloadError, entry, short, source, word,
};

#[test]
fn every_deflate_prefix_and_appended_byte_rejects_without_partial_publication() {
    let mut decoder = ZipPayloadDecoder::new();
    for end in 0..HELLO.len() {
        let fixture = source(&HELLO[..end], 5, HELLO_CRC, 8);
        let mut output = [0xa5; 5];
        assert!(
            decoder
                .decode(entry(&fixture.bytes), BUDGET, &mut [0; 6], &mut output)
                .is_err()
        );
        assert_eq!(output, [0xa5; 5]);
    }
    for byte in 0..=255 {
        let mut packed = HELLO.to_vec();
        packed.push(byte);
        let fixture = source(&packed, 5, HELLO_CRC, 8);
        let mut output = [0xa5; 5];
        assert_eq!(
            decoder
                .decode(entry(&fixture.bytes), BUDGET, &mut [0; 6], &mut output)
                .err(),
            Some(ZipPayloadError::TrailingInput)
        );
        assert_eq!(output, [0xa5; 5]);
    }
}

#[test]
fn codec_and_dirty_workspace_reuse_cannot_reference_a_previous_entry() {
    let good = source(HELLO, 5, HELLO_CRC, 8);
    let bad = source(HELLO, 5, 1, 8);
    let empty = source(b"\x03\0", 0, 0, 8);
    let mut decoder = ZipPayloadDecoder::new();
    let mut scratch = [0xff; 6];
    for _ in 0..16 {
        let mut output = [0xa5; 5];
        assert_eq!(
            decoder
                .decode(entry(&bad.bytes), BUDGET, &mut scratch, &mut output)
                .err(),
            Some(ZipPayloadError::Checksum)
        );
        assert_eq!(output, [0xa5; 5]);
        scratch.fill(0x92);
        assert!(
            decoder
                .decode(entry(&empty.bytes), BUDGET, &mut scratch, &mut output)
                .expect("Empty reset")
                .bytes()
                .is_empty()
        );
        assert_eq!(
            decoder
                .decode(entry(&good.bytes), BUDGET, &mut scratch, &mut output)
                .expect("Fresh entry")
                .bytes(),
            b"hello"
        );
    }
}

#[test]
fn stored_corruption_is_detected_and_catalog_metadata_does_not_imply_integrity() {
    let fixture = source(b"hello", 5, HELLO_CRC, 0);
    for offset in fixture.locals[0] + 30 + b"PrivateEntry".len()..fixture.central[0] {
        let mut bytes = fixture.bytes.clone();
        bytes[offset] ^= 1;
        let mut output = [0xa5; 5];
        assert_eq!(
            ZipPayloadDecoder::new()
                .decode(entry(&bytes), BUDGET, &mut [], &mut output)
                .err(),
            Some(ZipPayloadError::Checksum)
        );
        assert_eq!(output, [0xa5; 5]);
    }
    // Mutually consistent local/central CRC fields still need payload verification.
    // An archive comment is inert and cannot change the extraction result.
    let mut bytes = fixture.bytes.clone();
    let comment = b"PrivateComment";
    short(
        &mut bytes,
        fixture.end + 20,
        u16::try_from(comment.len()).expect("Comment"),
    );
    bytes.extend_from_slice(comment);
    word(&mut bytes, fixture.locals[0] + 14, 0);
    word(&mut bytes, fixture.central[0] + 16, 0);
    assert_eq!(
        ZipPayloadDecoder::new()
            .decode(entry(&bytes), BUDGET, &mut [], &mut [0xa5; 5])
            .err(),
        Some(ZipPayloadError::Checksum)
    );
}

#[test]
fn malformed_deflate_mutations_never_publish_partial_or_incorrect_payloads() {
    let mut decoder = ZipPayloadDecoder::new();
    for offset in 0..HELLO.len() {
        for mask in [1, 2, 4, 8, 16, 32, 64, 128, 255] {
            let mut packed = HELLO.to_vec();
            packed[offset] ^= mask;
            let fixture = source(&packed, 5, HELLO_CRC, 8);
            let mut output = [0xa5; 8];
            match decoder.decode(entry(&fixture.bytes), BUDGET, &mut [0xff; 6], &mut output) {
                Ok(checked) => assert_eq!(checked.bytes(), b"hello"),
                Err(_) => assert_eq!(output, [0xa5; 8]),
            }
            assert_eq!(&output[5..], &[0xa5; 3]);
        }
    }
}
