// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    AMMO_VECTOR, BodyReader, PROFILES, PlayerAmmunition, PlayerArrayError, PlayerStats,
    STAT_VECTOR, WeaponModels, encoded, parse_ammo, parse_models, parse_stats, position,
    write_ammo, write_stats,
};
use std::fmt::Debug;

fn truncations<T: Clone + Debug + Eq>(
    bytes: &[u8],
    prior: &T,
    parse: impl Fn(
        &mut BodyReader<'_>,
        super::BaselineProtocol,
        Option<&T>,
    ) -> Result<T, PlayerArrayError>,
) {
    let saved = prior.clone();
    for length in 0..bytes.len() {
        let mut reader = BodyReader::try_new(&bytes[..length], length).expect("Prefix body");
        if length != 0 {
            assert_eq!(reader.read_bits(3).expect("Cached prefix"), 5);
        }
        let before = position(&reader);
        assert!(
            parse(&mut reader, PROFILES[1], Some(prior)).is_err(),
            "prefix {length}"
        );
        assert_eq!(position(&reader), before);
        assert_eq!(prior, &saved);
    }
}

#[test]
fn every_stats_truncation_preserves_prior_and_cached_reader_bits() {
    let prior = parse_stats(PROFILES[1], &STAT_VECTOR, None);
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Prefix");
        write_stats(writer, 31, &[17, -123, 0, 63, 255]);
    });
    truncations(&bytes, &prior, PlayerStats::parse);
}

#[test]
fn every_ammunition_truncation_including_last_clip_preserves_both_original_owners() {
    let prior = parse_ammo(PROFILES[1], &AMMO_VECTOR, None);
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Prefix");
        write_ammo(writer, [0xffff; 4], &[0xffff; 8], &[17; 64], &[-32768; 128]);
    });
    truncations(&bytes, &prior, PlayerAmmunition::parse);
    // The final absent clip-group flag needs a new bit byte after its payload.
    let mut reader = BodyReader::try_new(&[2, 0, 0x80, 0xff, 0xff], 5).expect("Missing final flag");
    let before = position(&reader);
    assert!(PlayerAmmunition::parse(&mut reader, PROFILES[1], Some(&prior)).is_err());
    assert_eq!(position(&reader), before);
}

#[test]
fn every_model_table_truncation_preserves_all_prior_bytes_and_cached_bits() {
    let mut raw = vec![1];
    raw.extend([0xff; 128]);
    let prior = parse_models(PROFILES[1], &raw, None);
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Prefix");
        writer.write_bits(1, 1).expect("Changed models");
        for _ in 0..128 {
            writer.write_u8(0xfe).expect("Model byte");
        }
    });
    truncations(&bytes, &prior, WeaponModels::parse);
}

#[test]
fn all_section_reference_profiles_reject_before_empty_or_cached_reads() {
    for profile in PROFILES {
        let other = if profile == PROFILES[0] {
            PROFILES[1]
        } else {
            PROFILES[0]
        };
        for bytes in [&[][..], &[0xff][..]] {
            let mut reader = BodyReader::try_new(bytes, bytes.len()).expect("Body");
            if !bytes.is_empty() {
                reader.read_bits(3).expect("Cache");
            }
            let before = position(&reader);
            assert_eq!(
                PlayerStats::parse(&mut reader, profile, Some(&PlayerStats::zero(other))),
                Err(PlayerArrayError::ReferenceProfile)
            );
            assert_eq!(
                PlayerAmmunition::parse(&mut reader, profile, Some(&PlayerAmmunition::zero(other))),
                Err(PlayerArrayError::ReferenceProfile)
            );
            assert_eq!(
                WeaponModels::parse(&mut reader, profile, Some(&WeaponModels::zero(other))),
                Err(PlayerArrayError::ReferenceProfile)
            );
            assert_eq!(position(&reader), before);
        }
    }
}

#[test]
fn owned_projections_outlive_inputs_and_prior_state_and_hide_values_in_debug() {
    let stats = {
        let bytes = STAT_VECTOR.to_vec();
        let prior = parse_stats(PROFILES[1], &bytes, None);
        parse_stats(PROFILES[1], &[0], Some(&prior))
    };
    let ammo = {
        let bytes = AMMO_VECTOR.to_vec();
        let prior = parse_ammo(PROFILES[1], &bytes, None);
        parse_ammo(PROFILES[1], &[0, 0], Some(&prior))
    };
    let models = {
        let mut bytes = vec![1];
        bytes.extend([0xff; 128]);
        let prior = parse_models(PROFILES[1], &bytes, None);
        parse_models(PROFILES[1], &[0], Some(&prior))
    };
    assert_eq!(stats.get(0), Some(-32768));
    assert_eq!(ammo.clip(127), Some(-123));
    assert_eq!(models.get(127), Some(255));
    let debug = format!("{stats:?} {ammo:?} {models:?}");
    for private in ["-32768", "-123", "255"] {
        assert!(!debug.contains(private));
    }
}
