// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, PROFILES, PlayerScalarError, PlayerScalarPrefix, PlayerScalars, UNCONDITIONAL,
    encoded, header, index, integer, parse, position, seed, word,
};

#[test]
fn every_invalid_count_rejects_before_field_reads_and_preserves_both_cursors() {
    for count in 142..256 {
        let bytes = encoded(|writer| header(writer, true, count));
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let before = position(&reader);
        assert_eq!(
            PlayerScalarPrefix::parse_live(&mut reader, PROFILES[1], 0, None).err(),
            Some(PlayerScalarError::FieldCount)
        );
        assert_eq!(position(&reader), before);
    }
}

#[test]
fn every_truncated_full_prefix_preserves_cached_bits_and_all_prior_words() {
    let prior = seed(PROFILES[1], word::weapons::ADS_DELAY_TIME, 32, 0x89ab_cdef);
    let saved = prior.clone();
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Cached prefix");
        header(writer, false, 141);
        for &key in &word::ALL {
            if key == word::weapons::ADS_DELAY_TIME {
                integer(writer, 32, 0x89ab_cdef, 0x1234_5678, true);
            } else if let Some(&(_, width)) = UNCONDITIONAL.iter().find(|&&(field, _)| field == key)
            {
                integer(writer, width, 0, 1, false);
            } else {
                writer.write_bits(0, 1).expect("Unchanged");
            }
        }
    });
    for length in 1..bytes.len() {
        let mut reader = BodyReader::try_new(&bytes[..length], length).expect("Prefix body");
        assert_eq!(reader.read_bits(3).expect("Cached bits"), 5);
        let before = position(&reader);
        assert!(
            PlayerScalarPrefix::parse_live(&mut reader, PROFILES[1], 0, Some(&prior)).is_err(),
            "prefix {length}"
        );
        assert_eq!(position(&reader), before);
        assert_eq!(prior, saved);
    }
}

#[test]
fn invalid_compact_references_reject_late_without_publishing_earlier_fields() {
    for bits in [
        0x7fc0_1234,
        f32::INFINITY.to_bits(),
        4096.0_f32.to_bits(),
        (-4097.0_f32).to_bits(),
    ] {
        let key = word::effects::DOF_NEAR_BLUR;
        let prior = seed(PROFILES[1], key, 0, bits);
        let saved = prior.clone();
        let bytes = encoded(|writer| {
            header(writer, false, index(key) + 1);
            for &field in &word::ALL[..=index(key)] {
                if field == word::session::COMMAND_TIME {
                    writer.write_bits(3, 2).expect("Changed absolute time");
                    writer.write_i32(17).expect("Time");
                } else if field == key {
                    writer.write_bits(3, 3).expect("Changed compact float");
                    writer.write_bits(0, 5).expect("Low window");
                    writer.write_u8(0).expect("High window");
                } else {
                    writer.write_bits(0, 1).expect("Unchanged");
                }
            }
        });
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let before = position(&reader);
        assert_eq!(
            PlayerScalarPrefix::parse_live(&mut reader, PROFILES[1], 0, Some(&prior)).err(),
            Some(PlayerScalarError::ReferenceFloat)
        );
        assert_eq!(position(&reader), before);
        assert_eq!(prior, saved);
    }
}

#[test]
fn profile_binding_precedes_empty_reads_and_owned_projections_hide_values_in_debug() {
    for profile in PROFILES {
        let other = if profile == PROFILES[0] {
            PROFILES[1]
        } else {
            PROFILES[0]
        };
        let mut reader = BodyReader::try_new(&[], 0).expect("Empty body");
        assert_eq!(
            PlayerScalarPrefix::parse_live(
                &mut reader,
                profile,
                0,
                Some(&PlayerScalars::zero(other))
            )
            .err(),
            Some(PlayerScalarError::ReferenceProfile)
        );
        assert!(PlayerScalarPrefix::parse_live(&mut reader, profile, 0, None).is_err());
        assert_eq!(position(&reader), (0, 0, 0));
    }
    let owned = {
        let prior = seed(PROFILES[1], word::weapons::ADS_DELAY_TIME, 32, 0xdead_beef);
        let bytes = vec![0, 0];
        parse(PROFILES[1], 0, &bytes, Some(&prior))
    };
    assert_eq!(
        owned.scalars().word(word::weapons::ADS_DELAY_TIME),
        0xdead_beef
    );
    let debug = format!("{owned:?} {:?}", owned.scalars());
    for private in ["dead", "beef", "3735928559"] {
        assert!(!debug.contains(private));
    }
}
