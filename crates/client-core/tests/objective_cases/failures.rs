// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, FIELDS, ObjectiveError, PROFILES, SnapshotObjectives, W, encoded, integer,
    position, raw_float, seed,
};

#[test]
fn every_truncated_full_section_preserves_prior_and_cached_reader_bits() {
    for profile in PROFILES {
        let prior = seed(profile, 0xa5a5_a5a5);
        let saved = prior.clone();
        let bytes = encoded(|writer| {
            writer.write_bits(5, 3).expect("Existing cached bits");
            writer.write_bits(1, 1).expect("Section changed");
            for slot in 0..16 {
                writer.write_bits(7, 3).expect("Status");
                writer.write_bits(1, 1).expect("Entry changed");
                for &(word, width) in &FIELDS {
                    let old = prior.get(slot).expect("Prior slot").word(word);
                    if width == 0 {
                        raw_float(writer, old, 0x7fc0_4321);
                    } else {
                        integer(writer, width, old, u32::MAX >> (32 - width));
                    }
                }
            }
        });
        for length in 1..bytes.len() {
            let mut reader = BodyReader::try_new(&bytes[..length], length).expect("Truncation");
            assert_eq!(reader.read_bits(3).expect("Prefix"), 5);
            let before = position(&reader);
            assert!(
                SnapshotObjectives::parse(&mut reader, profile, Some(&prior)).is_err(),
                "prefix {length}"
            );
            assert_eq!(position(&reader), before);
            assert_eq!(prior, saved);
        }
    }
}

#[test]
fn a_late_invalid_compact_float_reference_rolls_back_all_earlier_slots() {
    for bits in [
        0x7fc0_1234,
        f32::INFINITY.to_bits(),
        4096.0_f32.to_bits(),
        (-4097.0_f32).to_bits(),
    ] {
        let prior = seed(PROFILES[1], bits);
        let saved = prior.clone();
        let bytes = encoded(|writer| {
            writer.write_bits(1, 1).expect("Changed section");
            for slot in 0..16 {
                writer.write_bits(5, 3).expect("New status");
                writer.write_bits(1, 1).expect("Entry changed");
                if slot == 15 {
                    writer.write_bits(3, 3).expect("Changed nonzero compact X");
                    writer.write_bits(0, 5).expect("Compact residual");
                    writer.write_u8(0).expect("Compact byte");
                } else {
                    raw_float(
                        writer,
                        prior.get(slot).expect("Prior").word(W::OriginX),
                        1.5_f32.to_bits(),
                    );
                }
                for _ in 1..6 {
                    writer.write_bits(0, 1).expect("Unchanged field");
                }
            }
        });
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let before = position(&reader);
        assert_eq!(
            SnapshotObjectives::parse(&mut reader, PROFILES[1], Some(&prior)),
            Err(ObjectiveError::ReferenceFloat)
        );
        assert_eq!(position(&reader), before);
        assert_eq!(prior, saved);
    }
}

#[test]
fn profile_mismatch_rejects_before_reading_an_empty_or_cached_body() {
    for profile in PROFILES {
        let other = if profile == PROFILES[0] {
            PROFILES[1]
        } else {
            PROFILES[0]
        };
        let prior = SnapshotObjectives::zero(other);
        for bytes in [&[][..], &[0xff][..]] {
            let mut reader = BodyReader::try_new(bytes, bytes.len()).expect("Body");
            let before = position(&reader);
            assert_eq!(
                SnapshotObjectives::parse(&mut reader, profile, Some(&prior)),
                Err(ObjectiveError::ReferenceProfile)
            );
            assert_eq!(position(&reader), before);
        }
    }
}

#[test]
fn an_empty_body_cannot_publish_a_zero_section() {
    let mut reader = BodyReader::try_new(&[], 0).expect("Empty");
    assert!(SnapshotObjectives::parse(&mut reader, PROFILES[1], None).is_err());
    assert_eq!(position(&reader), (0, 0, 0));
}
