// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, PLAYER_SCALAR_WORDS, PROFILES, PlayerScalarPrefix, PlayerScalars, encoded, header,
    integer, parse, word,
};
use std::collections::HashSet;

#[test]
fn closed_key_catalog_contains_all_141_distinct_semantic_fields() {
    assert_eq!(PLAYER_SCALAR_WORDS, 141);
    assert_eq!(word::ALL.len(), 141);
    assert_eq!(word::ALL.into_iter().collect::<HashSet<_>>().len(), 141);
    assert_eq!(word::ALL[0], word::session::COMMAND_TIME);
    assert_eq!(
        &word::ALL[1..4],
        &[
            word::view::ANGLE_Y,
            word::view::ANGLE_X,
            word::view::ANGLE_Z
        ]
    );
    assert_eq!(word::ALL[140], word::weapons::ADS_DELAY_TIME);
}

#[test]
fn physical_empty_prefix_modes_preserve_zero_and_mark_prediction_requests() {
    for profile in PROFILES {
        for origins in [false, true] {
            let result = parse(profile, 0, &[u8::from(origins), 0], None);
            assert_eq!(result.changed_fields(), 0);
            assert_eq!(result.origins_transmitted(), origins);
            assert_eq!(result.needs_prediction(), !origins);
            assert_eq!(result.scalars(), &PlayerScalars::zero(profile));
        }
    }
}

#[test]
fn independent_command_time_vector_uses_bit_count_and_wrapping_message_time() {
    for profile in PROFILES {
        for first in [2, 3] {
            let result = parse(profile, 16, &[first, 0x2a, 1], None);
            assert_eq!(result.changed_fields(), 1);
            assert_eq!(
                result.scalars().word(word::session::COMMAND_TIME),
                0xffff_ffeb
            );
            assert_eq!(result.origins_transmitted(), first == 3);
        }
    }
}

#[test]
fn independent_angle_vector_uses_native_y_x_z_order_and_signed_short_angles() {
    for profile in PROFILES {
        let result = parse(profile, 0, &[8, 0x1c, 0, 0x80, 0, 0x40, 0, 0xc0], None);
        assert_eq!(
            result.scalars().word(word::view::ANGLE_Y),
            (-180.0_f32).to_bits()
        );
        assert_eq!(
            result.scalars().word(word::view::ANGLE_X),
            90.0_f32.to_bits()
        );
        assert_eq!(
            result.scalars().word(word::view::ANGLE_Z),
            (-90.0_f32).to_bits()
        );
    }
}

#[test]
fn independent_negative_short_vector_sign_extends_weapon_time() {
    for profile in PROFILES {
        assert_eq!(
            parse(profile, 0, &[0x1c, 0, 0xc0, 0, 0x80], None)
                .scalars()
                .word(word::weapons::TIME),
            0xffff_8000
        );
        assert_eq!(
            parse(profile, 0, &[0x1c, 0, 0xc0, 0xff, 0xff], None)
                .scalars()
                .word(word::weapons::TIME),
            u32::MAX
        );
    }
}

#[test]
fn full_zero_prefix_covers_every_field_including_unconditional_encodings() {
    for profile in PROFILES {
        let bytes = encoded(|writer| {
            header(writer, true, 141);
            writer.write_bits(0, 32).expect("Zeros");
            for _ in 32..141 {
                writer.write_bits(0, 1).expect("Zero field");
            }
        });
        let result = parse(profile, 0, &bytes, None);
        assert_eq!(result.changed_fields(), 141);
        assert_eq!(result.into_scalars(), PlayerScalars::zero(profile));
    }
}

#[test]
fn existing_cached_bits_and_following_byte_fields_survive_complete_prefix() {
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Prefix");
        header(writer, false, 14);
        for _ in 0..13 {
            writer.write_bits(0, 1).expect("Unchanged");
        }
        integer(writer, 16, 0, 0xffff_8000, true);
        writer.write_u8(0xa7).expect("Following byte");
        writer.write_bits(3, 2).expect("Following cached bits");
    });
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    assert_eq!(reader.read_bits(3).expect("Prefix"), 5);
    let result = PlayerScalarPrefix::parse_live(&mut reader, PROFILES[1], 0, None).expect("Prefix");
    assert_eq!(result.scalars().word(word::weapons::TIME), 0xffff_8000);
    assert_eq!(reader.read_u8().expect("Following byte"), 0xa7);
    assert_eq!(reader.read_bits(2).expect("Following bits"), 3);
}
