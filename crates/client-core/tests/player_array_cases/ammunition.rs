// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{AMMO_VECTOR, BodyReader, PROFILES, PlayerAmmunition, encoded, parse_ammo, write_ammo};

#[test]
fn independent_physical_ammo_vector_covers_signed_values_high_masks_and_last_indices() {
    for profile in PROFILES {
        let result = parse_ammo(profile, &AMMO_VECTOR, None);
        assert_eq!(result.profile(), profile);
        assert_eq!(result.stored(0), Some(-32768));
        assert_eq!(result.stored(15), Some(-1));
        assert_eq!(result.stored(63), Some(32767));
        assert_eq!(result.clip(127), Some(-123));
        assert_eq!(result.stored(64), None);
        assert_eq!(result.clip(128), None);
        assert_eq!(result.stored_values().len(), 64);
        assert_eq!(result.clip_values().len(), 128);
        assert!(
            result
                .stored_values()
                .iter()
                .enumerate()
                .all(|(i, &v)| [0, 15, 63].contains(&i) || v == 0)
        );
        assert!(result.clip_values()[..127].iter().all(|&v| v == 0));
    }
}

#[test]
fn clips_have_no_overall_change_bit_and_keep_stored_ammo_unchanged() {
    for profile in PROFILES {
        let prior = parse_ammo(profile, &AMMO_VECTOR, None);
        let result = parse_ammo(profile, &[2, 0, 0x80, 0xff, 0xff, 0], Some(&prior));
        assert_eq!(result.stored_values(), prior.stored_values());
        assert_eq!(result.clip(15), Some(-1));
        assert_eq!(result.clip(127), Some(-123));
    }
}

#[test]
fn explicit_absolute_zero_values_clear_nonzero_stored_and_clip_counts() {
    for profile in PROFILES {
        let prior = parse_ammo(profile, &AMMO_VECTOR, None);
        let bytes = encoded(|writer| {
            write_ammo(
                writer,
                [0x8001, 0, 0, 0x8000],
                &[0, 0, 0, 0, 0, 0, 0, 0x8000],
                &[0; 64],
                &[0; 128],
            );
        });
        assert_eq!(
            parse_ammo(profile, &bytes, Some(&prior)),
            PlayerAmmunition::zero(profile)
        );
    }
}

#[test]
fn all_twelve_groups_apply_absolute_short_values_and_preserve_unset_entries() {
    for profile in PROFILES {
        let prior = parse_ammo(profile, &AMMO_VECTOR, None);
        let stored = std::array::from_fn(|i| i32::try_from(i).expect("Index") - 32);
        let clips = std::array::from_fn(|i| i32::try_from(i).expect("Index") - 64);
        let bytes =
            encoded(|writer| write_ammo(writer, [0xffff; 4], &[0xffff; 8], &stored, &clips));
        let result = parse_ammo(profile, &bytes, Some(&prior));
        assert_eq!(result.stored_values(), &stored);
        assert_eq!(result.clip_values(), &clips);
        let bytes =
            encoded(|writer| write_ammo(writer, [1, 0, 0, 0], &[0; 8], &[17; 64], &[99; 128]));
        let next = parse_ammo(profile, &bytes, Some(&result));
        assert_eq!(next.stored(0), Some(17));
        assert_eq!(&next.stored_values()[1..], &stored[1..]);
        assert_eq!(next.clip_values(), &clips);
    }
}

#[test]
fn unchanged_and_changed_zero_mask_groups_preserve_complete_prior_projection() {
    for profile in PROFILES {
        let prior = parse_ammo(profile, &AMMO_VECTOR, None);
        assert_eq!(parse_ammo(profile, &[0, 0], Some(&prior)), prior);
        // Stored section present, four present groups with empty masks, then eight
        // absent clip groups. Empty masks still consume their two whole bytes.
        let bytes = encoded(|writer| {
            writer.write_bits(1, 1).expect("Stored section");
            for _ in 0..4 {
                writer.write_bits(1, 1).expect("Present group");
                writer.write_i16(0).expect("Empty mask");
            }
            writer.write_bits(0, 8).expect("Eight absent clip groups");
        });
        assert_eq!(parse_ammo(profile, &bytes, Some(&prior)), prior);
        assert_eq!(
            parse_ammo(profile, &bytes, None),
            PlayerAmmunition::zero(profile)
        );
    }
}

#[test]
fn ammunition_preserves_existing_cached_bits_and_following_fields() {
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Prefix");
        write_ammo(
            writer,
            [0x8001, 0, 0, 0x8000],
            &[0, 0, 0, 0, 0, 0, 0, 0x8000],
            &[-32768; 64],
            &[-123; 128],
        );
        writer.write_u8(0xa7).expect("Following byte");
        writer.write_bits(3, 2).expect("Following bits");
    });
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    assert_eq!(reader.read_bits(3).expect("Prefix"), 5);
    let result = PlayerAmmunition::parse(&mut reader, PROFILES[1], None).expect("Ammo");
    assert_eq!(result.stored(63), Some(-32768));
    assert_eq!(result.clip(127), Some(-123));
    assert_eq!(reader.read_u8().expect("Following byte"), 0xa7);
    assert_eq!(reader.read_bits(2).expect("Following bits"), 3);
}
