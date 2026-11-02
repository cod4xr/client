// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{PROFILES, integer, one_field, raw_float, seed, word};

#[test]
fn signed_short_fields_mask_negative_priors_before_xor_and_sign_extend_targets() {
    let keys = [
        word::weapons::TIME,
        word::weapons::DELAY,
        word::motion::MOVE_TIME,
        word::weapons::GRENADE_TIME_LEFT,
        word::weapons::THROW_BACK_GRENADE_TIME_LEFT,
        word::weapons::RESTRICT_KICK_TIME,
    ];
    for profile in PROFILES {
        for key in keys {
            let prior = seed(profile, key, -16, u32::MAX);
            for target in [17, 0xffff_8000, 0] {
                let result = one_field(profile, 0, false, Some(&prior), key, |writer| {
                    integer(writer, 16, u32::MAX, target, true);
                });
                assert_eq!(result.scalars().word(key), target);
            }
        }
    }
}

#[test]
fn nonforced_raw_float_fields_keep_complete_prior_xor_in_either_origin_mode() {
    let keys = [
        word::weapons::AIM_SPREAD_SCALE,
        word::weapons::HOLD_BREATH_SCALE,
        word::weapons::POSITION_FRACTION,
        word::view::HEIGHT_CURRENT,
    ];
    for profile in PROFILES {
        for key in keys {
            let prior = seed(profile, key, -88, 1.5_f32.to_bits());
            for origins in [false, true] {
                let result = one_field(profile, 0, origins, Some(&prior), key, |writer| {
                    raw_float(writer, 1.5_f32.to_bits(), 0x7fc0_4321, false);
                });
                assert_eq!(result.scalars().word(key), 0x7fc0_4321);
            }
        }
    }
}

#[test]
fn ordinary_float_forms_distinguish_raw_signed_zero_and_compact_truncated_prior() {
    for (old, payload, expected) in [
        (0x7fc0_1234, &[0][..], 0),
        (0x7fc0_1234, &[2][..], 0x8000_0000),
        (1.75_f32.to_bits(), &[0x7d, 0xff][..], (-2.0_f32).to_bits()),
        ((-1.75_f32).to_bits(), &[0x75, 0xff][..], 2.0_f32.to_bits()),
        (
            1.5_f32.to_bits(),
            &[3, 0x34, 0x12, 0, 0x40][..],
            0x7fc0_1234,
        ),
    ] {
        let prior = seed(PROFILES[1], word::effects::DOF_NEAR_BLUR, 0, old);
        let result = one_field(
            PROFILES[1],
            0,
            false,
            Some(&prior),
            word::effects::DOF_NEAR_BLUR,
            |writer| {
                writer.write_bits(1, 1).expect("Changed float");
                writer
                    .write_bits(
                        u32::from(payload[0]),
                        if payload[0] & 3 == 1 { 7 } else { 2 },
                    )
                    .expect("Independent float bits");
                for &byte in &payload[1..] {
                    writer.write_u8(byte).expect("Independent float bytes");
                }
            },
        );
        assert_eq!(
            result.scalars().word(word::effects::DOF_NEAR_BLUR),
            expected
        );
    }
}

#[test]
fn view_angles_are_absolute_signed_shorts_without_a_zero_value_marker() {
    for key in [
        word::view::ANGLE_X,
        word::view::ANGLE_Y,
        word::view::ANGLE_Z,
    ] {
        let prior = one_field(PROFILES[1], 0, false, None, key, |writer| {
            writer.write_bits(1, 1).expect("Changed angle");
            writer.write_i16(0x4000).expect("Angle");
        })
        .into_scalars();
        for (short, expected) in [(0, 0.0_f32), (0xc000_u16.cast_signed(), -90.0)] {
            let result = one_field(PROFILES[1], 0, false, Some(&prior), key, |writer| {
                writer.write_bits(1, 1).expect("Changed angle");
                writer.write_i16(short).expect("Absolute angle");
            });
            assert_eq!(result.scalars().word(key), expected.to_bits());
        }
    }
}

#[test]
fn delta_angles_keep_their_separate_zero_marker_and_signed_angle_rule() {
    for key in [
        word::motion::DELTA_ANGLE_X,
        word::motion::DELTA_ANGLE_Y,
        word::motion::DELTA_ANGLE_Z,
    ] {
        let prior = one_field(PROFILES[1], 0, false, None, key, |writer| {
            writer
                .write_bits(3, 2)
                .expect("Changed nonzero delta angle");
            writer
                .write_i16(0xc000_u16.cast_signed())
                .expect("Negative angle");
        })
        .into_scalars();
        assert_eq!(prior.word(key), (-90.0_f32).to_bits());
        let next = one_field(PROFILES[1], 0, false, Some(&prior), key, |writer| {
            writer.write_bits(1, 2).expect("Changed zero angle");
        });
        assert_eq!(next.scalars().word(key), 0);
    }
}

#[test]
fn scalar_event_words_use_integer_xor_while_full_flags_replace_absolutely() {
    for key in [word::events::EVENT_0, word::events::PARAMETER_3] {
        let prior = seed(PROFILES[1], key, 8, 0xa5);
        let result = one_field(PROFILES[1], 0, false, Some(&prior), key, |writer| {
            integer(writer, 8, 0xa5, 0x5a, true);
        });
        assert_eq!(result.scalars().word(key), 0x5a);
    }
    let key = word::session::ENTITY_FLAGS;
    let prior = one_field(PROFILES[1], 0, false, None, key, |writer| {
        writer.write_bits(1, 2).expect("Changed toggle flags");
        writer.write_bits(31, 5).expect("High bit");
    })
    .into_scalars();
    assert_eq!(prior.word(key), 0x8000_0000);
    let cleared = one_field(PROFILES[1], 0, false, Some(&prior), key, |writer| {
        writer.write_bits(1, 2).expect("Changed toggle flags");
        writer.write_bits(31, 5).expect("High bit");
    });
    assert_eq!(cleared.scalars().word(key), 0);
    let full = one_field(PROFILES[1], 0, false, Some(&prior), key, |writer| {
        writer.write_bits(3, 2).expect("Changed full flags");
        for byte in [0x56, 0x34, 0x12] {
            writer.write_u8(byte).expect("Absolute flag byte");
        }
    });
    assert_eq!(full.scalars().word(key), 0x0012_3456);
}

#[test]
fn all_relative_time_fields_wrap_or_replace_with_full_absolute_bits() {
    let keys = [
        word::session::COMMAND_TIME,
        word::motion::LAST_SPRINT_START,
        word::motion::LAST_SPRINT_END,
        word::effects::SHELLSHOCK_TIME,
        word::effects::FOLIAGE_SOUND_TIME,
        word::weapons::MELEE_CHARGE_TIME,
    ];
    for key in keys {
        let prior = one_field(PROFILES[1], 7, false, None, key, |writer| {
            writer.write_bits(1, 2).expect("Changed relative time");
            writer.write_bits(255, 8).expect("Age");
        })
        .into_scalars();
        assert_eq!(prior.word(key), 7_u32.wrapping_sub(255));
        let result = one_field(
            PROFILES[1],
            0xffff_fffe,
            false,
            Some(&prior),
            key,
            |writer| {
                writer.write_bits(3, 2).expect("Changed absolute time");
                writer
                    .write_i32(0x89ab_cdef_u32.cast_signed())
                    .expect("Raw time");
            },
        );
        assert_eq!(result.scalars().word(key), 0x89ab_cdef);
    }
}
