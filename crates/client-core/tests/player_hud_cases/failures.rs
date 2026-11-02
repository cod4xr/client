// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, FIELDS, HudError, MODERN, SnapshotHud, W, array, encoded, integer, legacy,
    position, raw_float, seed,
};

#[test]
fn every_truncated_full_section_preserves_cached_bits_and_both_prior_arrays() {
    let prior = seed(MODERN, &[1; 31], 0x7fc0_1234);
    let saved = prior.clone();
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Cached prefix");
        writer.write_bits(1, 1).expect("HUD changed");
        for _ in 0..2 {
            array(writer, 31, 39, |_, index, writer| match FIELDS[index].0 {
                W::Kind => integer(writer, 4, 1, 1),
                W::Duration => integer(writer, 32, 0, 0x89ab_cdef),
                _ => writer.write_bits(0, 1).expect("Unchanged field"),
            });
        }
    });
    for length in 1..bytes.len() {
        let mut reader = BodyReader::try_new(&bytes[..length], length).expect("Prefix body");
        assert_eq!(reader.read_bits(3).expect("Cache"), 5);
        let before = position(&reader);
        assert!(
            SnapshotHud::parse(&mut reader, MODERN, 0, Some(&prior)).is_err(),
            "prefix {length}"
        );
        assert_eq!(position(&reader), before);
        assert_eq!(prior, saved);
    }
}

#[test]
fn every_invalid_last_index_in_either_array_rejects_without_publishing_archival_data() {
    for last in 40..64 {
        for late in [false, true] {
            let bytes = encoded(|writer| {
                writer.write_bits(1, 1).expect("HUD changed");
                if late {
                    array(writer, 1, 39, |_, index, writer| {
                        if FIELDS[index].0 == W::Kind {
                            integer(writer, 4, 0, 1);
                        } else {
                            writer.write_bits(0, 1).expect("Unchanged field");
                        }
                    });
                }
                writer.write_bits(1, 5).expect("One invalid element");
                writer.write_bits(last, 6).expect("Invalid last index");
            });
            let prior = seed(MODERN, &[1], 0x7fc0_1234);
            let saved = prior.clone();
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
            let before = position(&reader);
            assert_eq!(
                SnapshotHud::parse(&mut reader, MODERN, 0, Some(&prior)),
                Err(HudError::FieldCount)
            );
            assert_eq!(position(&reader), before);
            assert_eq!(prior, saved);
        }
    }
}

#[test]
fn late_bad_compact_references_preserve_every_preceding_element_and_current_array() {
    for bits in [
        0x7fc0_1234,
        f32::INFINITY.to_bits(),
        2_147_483_648.0_f32.to_bits(),
        (-2_147_483_904.0_f32).to_bits(),
    ] {
        let prior = seed(MODERN, &[1; 31], bits);
        let saved = prior.clone();
        let bytes = encoded(|writer| {
            writer.write_bits(1, 1).expect("HUD changed");
            array(writer, 31, 39, |slot, index, writer| {
                match FIELDS[index].0 {
                    W::Kind => integer(writer, 4, 1, 1),
                    W::FromX if slot == 30 => {
                        writer.write_bits(3, 3).expect("Changed compact from-X");
                        writer.write_bits(0, 4).expect("Low window");
                        writer.write_u8(0).expect("High window");
                    }
                    W::FromX => raw_float(writer, bits, 1.5_f32.to_bits()),
                    _ => writer.write_bits(0, 1).expect("Unchanged field"),
                }
            });
            writer.write_bits(0, 5).expect("No current prefix");
        });
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let before = position(&reader);
        assert_eq!(
            SnapshotHud::parse(&mut reader, MODERN, 0, Some(&prior)),
            Err(HudError::ReferenceFloat)
        );
        assert_eq!(position(&reader), before);
        assert_eq!(prior, saved);
    }
}

#[test]
fn profile_mismatch_precedes_empty_reads_and_an_empty_body_never_becomes_zero_state() {
    let mut reader = BodyReader::try_new(&[], 0).expect("Empty body");
    let prior = SnapshotHud::zero(MODERN);
    assert_eq!(
        SnapshotHud::parse(&mut reader, legacy([0.0; 3]), 0, Some(&prior)),
        Err(HudError::ReferenceProfile)
    );
    assert!(SnapshotHud::parse(&mut reader, MODERN, 0, None).is_err());
    assert_eq!(position(&reader), (0, 0, 0));
}

#[test]
fn a_bad_last_current_element_cancels_valid_archival_changes_and_preserves_cached_bits() {
    let old = encoded(|writer| {
        writer.write_bits(1, 1).expect("HUD changed");
        for _ in 0..2 {
            array(writer, 31, 39, |_, index, writer| match FIELDS[index].0 {
                W::Kind => integer(writer, 4, 0, 1),
                W::FromX => raw_float(writer, 0, 0x7fc0_1234),
                _ => writer.write_bits(0, 1).expect("Unchanged"),
            });
        }
    });
    let prior = super::parse(MODERN, 0, &old, None);
    let saved = prior.clone();
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Cached prefix");
        writer.write_bits(1, 1).expect("HUD changed");
        array(writer, 1, 39, |_, index, writer| {
            if FIELDS[index].0 == W::Width {
                integer(writer, 10, 0, 17);
            } else {
                writer.write_bits(0, 1).expect("Unchanged");
            }
        });
        array(writer, 31, 39, |slot, index, writer| {
            if slot == 30 && FIELDS[index].0 == W::FromX {
                writer.write_bits(3, 3).expect("Changed compact from-X");
                writer.write_bits(0, 4).expect("Low window");
                writer.write_u8(0).expect("High window");
            } else {
                writer.write_bits(0, 1).expect("Unchanged");
            }
        });
    });
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    reader.read_bits(3).expect("Prefix");
    let before = position(&reader);
    assert_eq!(
        SnapshotHud::parse(&mut reader, MODERN, 0, Some(&prior)),
        Err(HudError::ReferenceFloat)
    );
    assert_eq!(position(&reader), before);
    assert_eq!(prior, saved);
}
