// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    FORCED, PROFILES, PlayerScalars, UNCONDITIONAL, encoded, header, index, integer, one_field,
    parse, raw_float, seed, word,
};

#[test]
fn all_eight_live_forced_fields_use_zero_references_while_retaining_changed_flags() {
    for profile in PROFILES {
        for &(key, bits) in &FORCED {
            let old = match bits {
                -88 => 1.5_f32.to_bits(),
                -8 => 0xffff_fffd,
                _ => 0xa5,
            };
            let value = match bits {
                -88 => 0x7fc0_1234,
                -8 => u32::MAX,
                _ => 17,
            };
            let prior = seed(profile, key, bits, old);
            let next = one_field(profile, 0, true, Some(&prior), key, |writer| {
                if bits == -88 {
                    raw_float(writer, 0, value, false);
                } else {
                    integer(
                        writer,
                        u8::try_from(bits.unsigned_abs()).expect("Width"),
                        0,
                        value,
                        true,
                    );
                }
            });
            assert_eq!(next.scalars().word(key), value);
            let unchanged = one_field(profile, 0, false, Some(&prior), key, |writer| {
                writer.write_bits(0, 1).expect("Unchanged");
            });
            assert_eq!(unchanged.scalars().word(key), old);
            let forced_zero = one_field(profile, 0, true, Some(&prior), key, |writer| {
                writer.write_bits(0, 1).expect("Forced reference copy");
            });
            assert_eq!(forced_zero.scalars().word(key), 0);
            assert_eq!(prior.word(key), old);
        }
    }
}

#[test]
fn origin_mode_does_not_clear_forced_fields_outside_the_serialized_prefix() {
    let prior = seed(PROFILES[1], word::motion::ORIGIN_X, -88, 1.5_f32.to_bits());
    let next = parse(PROFILES[1], 0, &[1, 0], Some(&prior));
    assert_eq!(next.changed_fields(), 0);
    assert!(next.origins_transmitted());
    assert_eq!(next.into_scalars(), prior);
}

#[test]
fn all_six_unconditional_fields_have_value_markers_without_outer_changed_bits() {
    for profile in PROFILES {
        for &(key, width) in &UNCONDITIONAL {
            let target = if width == 1 { 1 } else { 65535 };
            let result = one_field(profile, 0, false, None, key, |writer| {
                integer(writer, width, 0, target, false);
            });
            assert_eq!(result.scalars().word(key), target);
            let prior = result.into_scalars();
            let cleared = one_field(profile, 0, false, Some(&prior), key, |writer| {
                writer.write_bits(0, 1).expect("Unconditional zero marker");
            });
            assert_eq!(cleared.scalars().word(key), 0);
        }
    }
}

#[test]
fn unconditional_fields_are_retained_only_outside_prefix_and_explicitly_zeroed_inside_it() {
    for profile in PROFILES {
        let bytes = encoded(|writer| {
            header(writer, false, 141);
            for &key in &word::ALL {
                if let Some(&(_, width)) = UNCONDITIONAL.iter().find(|&&(field, _)| field == key) {
                    integer(writer, width, 0, 1, false);
                } else {
                    writer.write_bits(0, 1).expect("Unchanged");
                }
            }
        });
        let prior = parse(profile, 0, &bytes, None).into_scalars();
        assert_eq!(
            parse(profile, 0, &[0, 0], Some(&prior)).into_scalars(),
            prior
        );
        let bytes = encoded(|writer| {
            header(writer, false, 141);
            for _ in 0..141 {
                writer.write_bits(0, 1).expect("Zero marker");
            }
        });
        let next = parse(profile, 0, &bytes, Some(&prior)).into_scalars();
        assert_eq!(next, PlayerScalars::zero(profile));
    }
}

#[test]
fn final_hint_one_field_keeps_ordinary_changed_bit_and_complete_prior_xor() {
    assert_eq!(index(word::weapons::ADS_DELAY_TIME), 140);
    let prior = seed(PROFILES[1], word::weapons::ADS_DELAY_TIME, 32, 0x89ab_cdef);
    let next = one_field(
        PROFILES[1],
        0,
        false,
        Some(&prior),
        word::weapons::ADS_DELAY_TIME,
        |writer| {
            integer(writer, 32, 0x89ab_cdef, 0x1234_5678, true);
        },
    );
    assert_eq!(
        next.scalars().word(word::weapons::ADS_DELAY_TIME),
        0x1234_5678
    );
}
