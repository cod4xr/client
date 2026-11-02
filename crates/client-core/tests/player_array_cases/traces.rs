// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    PROFILES, PlayerAmmunition, PlayerStats, WeaponModels, encoded, parse_ammo, parse_models,
    parse_stats, write_ammo, write_stats,
};

fn random(value: &mut u32) -> u32 {
    *value = value.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    *value
}

#[test]
fn modeled_stats_traces_keep_every_omitted_value_and_apply_absolute_updates() {
    for profile in PROFILES {
        let mut result = PlayerStats::zero(profile);
        let mut expected = [0; 5];
        let mut generator = 0x7198_8e12;
        for _ in 0..128 {
            let mask = (random(&mut generator) >> 11) & 31;
            let values = std::array::from_fn(|slot| {
                let raw = random(&mut generator);
                match slot {
                    3 => i32::try_from(raw & 63).expect("Six bits"),
                    4 => i32::from(raw.to_le_bytes()[0]),
                    _ => i32::from(i16::from_le_bytes(
                        raw.to_le_bytes()[..2].try_into().expect("Short"),
                    )),
                }
            });
            for slot in 0..5 {
                if mask & (1 << slot) != 0 {
                    expected[slot] = values[slot];
                }
            }
            let bytes = encoded(|writer| write_stats(writer, mask, &values));
            result = parse_stats(profile, &bytes, Some(&result));
            assert_eq!(result.values(), &expected);
        }
    }
}

#[test]
fn modeled_ammunition_traces_preserve_unset_entries_across_every_group() {
    for profile in PROFILES {
        let mut result = PlayerAmmunition::zero(profile);
        let mut expected_stored = [0; 64];
        let mut expected_clips = [0; 128];
        let mut generator = 0x74a1_821d;
        for _ in 0..128 {
            let stored_masks =
                std::array::from_fn(|_| u16::try_from(random(&mut generator) >> 16).expect("Mask"));
            let clip_masks =
                std::array::from_fn(|_| u16::try_from(random(&mut generator) >> 16).expect("Mask"));
            let stored = std::array::from_fn(|_| {
                i32::from(i16::from_le_bytes(
                    random(&mut generator).to_le_bytes()[..2]
                        .try_into()
                        .expect("Short"),
                ))
            });
            let clips = std::array::from_fn(|_| {
                i32::from(i16::from_le_bytes(
                    random(&mut generator).to_le_bytes()[..2]
                        .try_into()
                        .expect("Short"),
                ))
            });
            for i in 0..64 {
                if stored_masks[i / 16] & (1 << (i % 16)) != 0 {
                    expected_stored[i] = stored[i];
                }
            }
            for i in 0..128 {
                if clip_masks[i / 16] & (1 << (i % 16)) != 0 {
                    expected_clips[i] = clips[i];
                }
            }
            let bytes =
                encoded(|writer| write_ammo(writer, stored_masks, &clip_masks, &stored, &clips));
            result = parse_ammo(profile, &bytes, Some(&result));
            assert_eq!(result.stored_values(), &expected_stored);
            assert_eq!(result.clip_values(), &expected_clips);
        }
    }
}

#[test]
fn modeled_model_tables_replace_completely_or_retain_all_prior_bytes() {
    for profile in PROFILES {
        let mut result = WeaponModels::zero(profile);
        let mut expected = [0; 128];
        let mut generator = 0x77a9_321d;
        for _ in 0..128 {
            let changed = !random(&mut generator).is_multiple_of(3);
            let values =
                std::array::from_fn::<_, 128, _>(|_| random(&mut generator).to_le_bytes()[0]);
            let bytes = encoded(|writer| {
                writer
                    .write_bits(u32::from(changed), 1)
                    .expect("Models changed");
                if changed {
                    for &value in &values {
                        writer.write_u8(value).expect("Model byte");
                    }
                }
            });
            if changed {
                expected = values;
            }
            result = parse_models(profile, &bytes, Some(&result));
            assert_eq!(result.values(), &expected);
        }
    }
}
