// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, HELLO, HELLO_CRC, ZipPayloadDecoder, entry, source};

#[test]
fn stored_and_all_deflate_block_forms_produce_only_whole_checked_bytes() {
    for (bytes, method) in [
        (&b"hello"[..], 0),
        (HELLO, 8),
        (&b"\x01\x05\0\xfa\xffhello"[..], 8),
    ] {
        let fixture = source(bytes, 5, HELLO_CRC, method);
        let mut output = [0xa5; 10];
        let mut scratch = [0x92; 12];
        let checked = ZipPayloadDecoder::new()
            .decode(entry(&fixture.bytes), BUDGET, &mut scratch, &mut output)
            .expect("Stored, fixed or stored-Deflate block");
        assert_eq!(checked.bytes(), b"hello");
        assert!(!format!("{checked:?}").contains("hello"));
        assert_eq!(&output[5..], &[0xa5; 5]);
        assert_eq!(&scratch[6..], &[0x92; 6]);
    }
}

#[test]
fn output_outlives_archive_metadata_decoder_and_scratch() {
    let mut output = [0; 5];
    let checked = {
        let fixture = source(HELLO, 5, HELLO_CRC, 8);
        let mut decoder = ZipPayloadDecoder::default();
        let checked = decoder
            .decode(entry(&fixture.bytes), BUDGET, &mut [0; 6], &mut output)
            .expect("Source-independent checked payload");
        assert_eq!(format!("{decoder:?}"), "ZipPayloadDecoder { .. }");
        checked
    };
    assert_eq!(checked.bytes(), b"hello");
}

#[test]
fn empty_stored_and_empty_deflate_have_exact_zero_crc_without_output_storage() {
    for (bytes, method) in [(&b""[..], 0), (&b"\x03\0"[..], 8)] {
        let fixture = source(bytes, 0, 0, method);
        let checked = ZipPayloadDecoder::new()
            .decode(entry(&fixture.bytes), BUDGET, &mut [0xff; 1], &mut [])
            .expect("Empty payload");
        assert!(checked.bytes().is_empty());
    }
    let fixture = source(b"hello", 5, HELLO_CRC, 0);
    ZipPayloadDecoder::new()
        .decode(entry(&fixture.bytes), BUDGET, &mut [], &mut [0; 5])
        .expect("Stored payload needs no codec scratch");
}
