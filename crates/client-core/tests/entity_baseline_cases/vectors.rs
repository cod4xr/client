// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{decoder, fixture};
use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4xr_client_core::{
    gamestate::entity_baselines::BaselineWord as W, message_body::BodyReader,
};

#[test]
fn independent_no_delta_and_type_vectors_preserve_following_opcode_and_cached_bits() {
    for (vector, kind, tail) in [(&[0xa1, 7][..], 0, 20), (&[0x0d, 0xae, 17, 7][..], 17, 21)] {
        for padding in [0, MAX_MESSAGE_BYTES + 7] {
            let baseline = {
                let mut input = vec![0xfe; padding];
                input.extend_from_slice(vector);
                let mut reader = BodyReader::try_new(&input, input.len()).expect("Complete body");
                reader.read_bytes(&mut vec![0; padding]).expect("Prefix");
                let baseline = decoder()
                    .parse(&mut reader)
                    .expect("Independent physical vector");
                assert_eq!(reader.read_u8(), Ok(7));
                assert_eq!(reader.read_bits(5), Ok(tail));
                assert_eq!(reader.remaining_bytes(), 0);
                baseline
            };
            assert_eq!(baseline.number(), 0);
            assert_eq!(baseline.word(W::Kind), kind);
            assert_eq!(baseline.word(W::PositionX), 0);
            assert_eq!(baseline.word(W::Part3), 0);
        }
    }
}

#[test]
fn every_layout_and_unknown_type_fallback_consumes_exactly_its_native_zero_fields() {
    for kind in (0_u8..18).chain([18, 255]) {
        let count = match kind {
            12 => 58,
            13 => 60,
            _ => 59,
        };
        let input = fixture(17, Some(count), kind, |writer| {
            for position in 1..count {
                // Independent native unconditional-field locations. All other fields
                // encode one outer unchanged bit, not an inner zero-value marker.
                match (kind, position) {
                    (1, 1 | 2) => writer.write_i32(0).expect("Unconditional raw origin"),
                    (3, 6 | 7 | 13 | 14) | (13, 1..=3 | 5 | 6) => {
                        writer.write_bits(0, 2).expect("Unconditional float zero");
                    }
                    (3, 12) | (13, 7) => writer
                        .write_bits(0, 9)
                        .expect("Unconditional relative time zero"),
                    _ => writer
                        .write_bits(0, 1)
                        .expect("Unchanged or unconditional integer zero"),
                }
            }
            writer.write_u8(7).expect("Following opcode");
            writer.write_bits(21, 5).expect("Following bit field");
        });
        let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
        let baseline = decoder().parse(&mut reader).expect("Full native layout");
        assert_eq!(baseline.number(), 17);
        assert_eq!(baseline.word(W::Kind), u32::from(kind));
        for word in [
            W::PositionX,
            W::PositionY,
            W::PositionZ,
            W::Union0,
            W::Union6,
            W::Part3,
        ] {
            assert_eq!(baseline.word(word), 0);
        }
        assert_eq!(reader.read_u8(), Ok(7));
        assert_eq!(reader.read_bits(5), Ok(21));
        assert_eq!(reader.remaining_bytes(), 0);
    }
}

#[test]
fn player_unconditional_coordinates_signed_union_and_angle_keep_exact_wire_words() {
    let input = fixture(1023, Some(6), 1, |writer| {
        writer
            .write_i32(0x7fc0_1234)
            .expect("Unconditional NaN payload bits");
        writer
            .write_i32(1_000_000.0_f32.to_bits().cast_signed())
            .expect("Large Y");
        writer
            .write_bits(3, 2)
            .expect("Changed, nonzero signed movement direction");
        writer.write_u8(0xfd).expect("Minus three");
        writer.write_bits(3, 2).expect("Changed nonzero angle");
        writer.write_i16(0x4000).expect("Ninety degrees");
        writer.write_bits(1, 1).expect("Changed Z");
        writer.write_i32(0x3f80_0000).expect("Z binary32");
        writer.write_u8(7).expect("Following opcode");
        writer.write_bits(21, 5).expect("Following bits");
    });
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
    let baseline = decoder().parse(&mut reader).expect("Type-one field order");
    assert_eq!(baseline.number(), 1023);
    assert_eq!(baseline.word(W::Kind), 1);
    assert_eq!(baseline.word(W::PositionX), 0x7fc0_1234);
    assert_eq!(baseline.word(W::PositionY), 1_000_000.0_f32.to_bits());
    assert_eq!(baseline.word(W::Union1), 0xffff_fffd);
    assert_eq!(baseline.word(W::AngleY), 90.0_f32.to_bits());
    assert_eq!(baseline.word(W::PositionZ), 1.0_f32.to_bits());
    assert_eq!(reader.read_u8(), Ok(7));
    assert_eq!(reader.read_bits(5), Ok(21));
    assert_eq!(
        format!("{baseline:?}"),
        "EntityBaseline { number: 1023, words: 60, .. }"
    );
}
