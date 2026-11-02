// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    BUDGET, HELLO, HELLO_CRC, ZipPayloadBudget, ZipPayloadDecoder, ZipPayloadError, entry, source,
};
use cod4xr_content::zip32::Zip32Compression;

#[test]
fn copied_metadata_routes_exact_bytes_and_outlives_all_original_archive_storage() {
    let (metadata, reread) = {
        let fixture = source(HELLO, 5, HELLO_CRC, 8);
        let value = entry(&fixture.bytes);
        let metadata = value.metadata();
        assert_eq!(metadata.data_offset(), 30 + b"PrivateEntry".len());
        let begin = metadata.data_offset();
        let end = begin + metadata.compressed_byte_count();
        assert_eq!(&fixture.bytes[begin..end], value.compressed_bytes());
        (metadata, fixture.bytes[begin..end].to_vec())
    };
    assert_eq!(metadata.compressed_byte_count(), 7);
    assert_eq!(metadata.declared_uncompressed_bytes(), 5);
    assert_eq!(metadata.declared_crc32(), HELLO_CRC);
    assert_eq!(metadata.compression(), Zip32Compression::Deflate);
    let debug = format!("{metadata:?}");
    for private in ["Private", "crc", "3610", "907060870", "hello"] {
        assert!(!debug.contains(private));
    }
    let mut output = [0xa5; 8];
    let checked = {
        let mut decoder = ZipPayloadDecoder::new();
        decoder
            .decode_bytes(metadata, &reread, BUDGET, &mut [0; 6], &mut output)
            .expect("Separately supplied payload bytes")
    };
    assert_eq!(checked.bytes(), b"hello");
    assert_eq!(&output[5..], &[0xa5; 3]);
}

#[test]
fn every_short_or_extra_source_rejects_before_any_final_output_or_scratch_is_needed() {
    let fixture = source(HELLO, 5, HELLO_CRC, 8);
    let metadata = entry(&fixture.bytes).metadata();
    let mut decoder = ZipPayloadDecoder::new();
    for length in 0..HELLO.len() {
        let mut output = [0xa5; 5];
        assert_eq!(
            decoder
                .decode_bytes(metadata, &HELLO[..length], BUDGET, &mut [], &mut output)
                .err(),
            Some(ZipPayloadError::InputLength)
        );
        assert_eq!(output, [0xa5; 5]);
    }
    for byte in 0..=255 {
        let mut input = HELLO.to_vec();
        input.push(byte);
        let mut output = [0xa5; 5];
        assert_eq!(
            decoder
                .decode_bytes(metadata, &input, BUDGET, &mut [], &mut output)
                .err(),
            Some(ZipPayloadError::InputLength)
        );
        assert_eq!(output, [0xa5; 5]);
    }
    // A short read cannot hide an over-budget catalog declaration.
    assert_eq!(
        decoder
            .decode_bytes(
                metadata,
                &[],
                ZipPayloadBudget {
                    max_compressed_bytes: 6,
                    ..BUDGET
                },
                &mut [],
                &mut []
            )
            .err(),
        Some(ZipPayloadError::InputBudget)
    );
}

#[test]
fn equal_sized_replacement_source_still_requires_the_original_payload_checksum() {
    let metadata = {
        let fixture = source(b"hello", 5, HELLO_CRC, 0);
        entry(&fixture.bytes).metadata()
    };
    let mut output = [0xa5; 5];
    assert_eq!(
        ZipPayloadDecoder::new()
            .decode_bytes(metadata, b"jello", BUDGET, &mut [], &mut output)
            .err(),
        Some(ZipPayloadError::Checksum)
    );
    assert_eq!(output, [0xa5; 5]);
    assert_eq!(
        ZipPayloadDecoder::new()
            .decode_bytes(metadata, b"hello", BUDGET, &mut [], &mut output)
            .expect("Original stored bytes")
            .bytes(),
        b"hello"
    );
}
