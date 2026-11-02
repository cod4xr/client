// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{MAX_MESSAGE_BYTES, padded, skip_padding, state};
use cod4x_protocol::{
    LegacyIndexDecoder, LegacyIndexEncoder, LegacyIndexError, LegacyIndexKind, MessageReader,
    MessageWriter,
};
use cod4xr_client_core::message_body::{BodyReadError, BodyReader};

#[test]
fn concrete_shared_index_decoders_preserve_cache_and_ordering_across_rebased_byte_fields() {
    for kind in [LegacyIndexKind::Entity, LegacyIndexKind::Client] {
        let mut output = [0; 96];
        let mut writer = MessageWriter::try_new(&mut output).expect("Field fixture");
        writer.write_bits(5, 3).expect("Cached prefix");
        writer.write_u8(0xaa).expect("Byte");
        let mut encoder = LegacyIndexEncoder::new(kind);
        for index in [0, 17, 20] {
            encoder.write(index, &mut writer).expect("Shared index");
            writer
                .write_i32(0x1234_5678)
                .expect("Intervening byte cursor");
        }
        encoder.reset();
        encoder.write(0, &mut writer).expect("New ordered stream");
        let bytes = writer.as_bytes();
        let input = padded(bytes, MAX_MESSAGE_BYTES * 2 + 3);
        let mut actual = BodyReader::try_new(&input, input.len()).expect("Body");
        skip_padding(&mut actual, MAX_MESSAGE_BYTES * 2 + 3);
        let mut reference = MessageReader::try_new(bytes).expect("Shared field view");
        let mut actual_indices = LegacyIndexDecoder::new(kind);
        let mut reference_indices = LegacyIndexDecoder::new(kind);
        assert_eq!(
            actual.read_bits(3).expect("Bits"),
            reference.read_bits(3).expect("Reference")
        );
        assert_eq!(
            actual.read_u8().expect("Byte"),
            reference.read_u8().expect("Reference")
        );
        for expected in [0, 17, 20] {
            assert_eq!(actual.read_index(&mut actual_indices), Ok(expected));
            assert_eq!(reference_indices.read(&mut reference), Ok(expected));
            assert_eq!(
                actual.read_i32().expect("Integer"),
                reference.read_i32().expect("Reference")
            );
            assert_eq!(actual.remaining_bytes(), reference.remaining_bytes());
        }
        actual_indices.reset();
        reference_indices.reset();
        assert_eq!(actual.read_index(&mut actual_indices), Ok(0));
        assert_eq!(reference_indices.read(&mut reference), Ok(0));
    }
}

#[test]
fn invalid_ordered_index_rolls_back_both_view_cache_and_the_external_shared_decoder() {
    let mut output = [0; 16];
    let mut writer = MessageWriter::try_new(&mut output).expect("Fixture");
    let mut encoder = LegacyIndexEncoder::new(LegacyIndexKind::Entity);
    encoder.write(10, &mut writer).expect("First index");
    writer
        .write_bits((7 << 2) | 2, 12)
        .expect("Independent lower absolute index");
    let mut actual = BodyReader::try_new(writer.as_bytes(), 16).expect("Body");
    let mut indices = LegacyIndexDecoder::new(LegacyIndexKind::Entity);
    assert_eq!(actual.read_index(&mut indices), Ok(10));
    let before = state(&actual);
    assert_eq!(
        actual.read_index(&mut indices),
        Err(BodyReadError::Index(LegacyIndexError::NotIncreasing))
    );
    assert_eq!(state(&actual), before);
    let mut next = MessageReader::try_new(&[1]).expect("Independent increment bit");
    assert_eq!(indices.read(&mut next), Ok(11));
    let mut fresh_stream = LegacyIndexDecoder::new(LegacyIndexKind::Entity);
    assert_eq!(actual.read_index(&mut fresh_stream), Ok(7));
}
