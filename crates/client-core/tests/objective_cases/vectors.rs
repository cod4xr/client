// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, FIELDS, OBJECTIVE_SLOTS, OBJECTIVE_WORDS, PROFILES, SnapshotObjectives, W, encoded,
    integer, parse, raw_float, seed,
};

#[test]
fn independent_physical_status_cycle_and_unknown_status_preserve_all_three_bits() {
    for profile in PROFILES {
        let result = parse(
            profile,
            &[0x21, 0x64, 0xa8, 0xec, 0x20, 0x64, 0xa8, 0xec, 0],
            None,
        );
        assert_eq!(result.profile(), profile);
        assert_eq!(result.objectives().len(), OBJECTIVE_SLOTS);
        for (slot, objective) in result.objectives().enumerate() {
            assert_eq!(usize::from(objective.status()), slot % 8);
            for &(word, _) in &FIELDS {
                assert_eq!(objective.word(word), 0);
            }
        }
        assert!(result.get(16).is_none());
        let reserved = parse(profile, &[15, 0, 0, 0, 0, 0, 0, 0, 0], None);
        assert_eq!(reserved.get(0).expect("First slot").status(), 7);
        assert_eq!(OBJECTIVE_WORDS, 6);
    }
}

#[test]
fn unchanged_section_returns_owned_prior_or_explicit_zero() {
    for profile in PROFILES {
        let prior = seed(profile, 0x7fc0_1234);
        assert_eq!(parse(profile, &[0], Some(&prior)), prior);
        assert_eq!(
            parse(profile, &[0], None),
            SnapshotObjectives::zero(profile)
        );
    }
}

#[test]
fn a_changed_entry_with_six_unchanged_flags_has_no_prefix_count() {
    let prior = seed(PROFILES[1], 0x7fc0_1234);
    let result = parse(PROFILES[1], &[0x19, 0, 0, 0, 0, 0, 0, 0, 0], Some(&prior));
    assert_eq!(result.get(0).expect("First status").status(), 4);
    for slot in 0..16 {
        for &(word, _) in &FIELDS {
            assert_eq!(
                result.get(slot).expect("Result").word(word),
                prior.get(slot).expect("Prior").word(word)
            );
        }
    }
}

#[test]
fn zero_forms_clear_all_words_even_from_nonfinite_float_references() {
    let prior = seed(PROFILES[1], 0x7fc0_1234);
    let bytes = encoded(|writer| {
        writer.write_bits(1, 1).expect("Section changed");
        for _ in 0..16 {
            writer.write_bits(0, 3).expect("Status");
            writer.write_bits(1, 1).expect("Entry changed");
            for &(_, width) in &FIELDS {
                if width == 0 {
                    writer
                        .write_bits(1, 3)
                        .expect("Changed positive zero float");
                } else {
                    integer(writer, width, 0, 0);
                }
            }
        }
    });
    assert_eq!(
        parse(PROFILES[1], &bytes, Some(&prior)),
        SnapshotObjectives::zero(PROFILES[1])
    );
}

#[test]
fn separate_status_updates_retain_every_prior_word_even_for_empty_status() {
    for profile in PROFILES {
        let prior = seed(profile, 0x7fc0_1234);
        let result = parse(
            profile,
            &[0x21, 0x64, 0xa8, 0xec, 0x20, 0x64, 0xa8, 0xec, 0],
            Some(&prior),
        );
        for slot in 0..16 {
            let old = prior.get(slot).expect("Old slot");
            let new = result.get(slot).expect("New slot");
            assert_eq!(usize::from(new.status()), slot % 8);
            for &(word, _) in &FIELDS {
                assert_eq!(new.word(word), old.word(word));
            }
        }
        // Clearing status alone retains words; engine interpretation is separate.
        let cleared = parse(profile, &[1, 0, 0, 0, 0, 0, 0, 0, 0], Some(&prior));
        assert!(
            cleared
                .objectives()
                .all(|objective| objective.status() == 0)
        );
        assert_eq!(cleared.get(0).expect("Slot").word(W::OriginX), 0x7fc0_1234);
    }
}

#[test]
fn all_six_fields_use_full_prior_xor_native_widths_and_distinct_semantic_order() {
    for profile in PROFILES {
        let prior = seed(profile, 0xa5a5_a5a5);
        let bytes = encoded(|writer| {
            writer.write_bits(1, 1).expect("Changed section");
            for slot in 0..16 {
                writer.write_bits(5, 3).expect("Status");
                writer.write_bits(1, 1).expect("Entry changed");
                let old = prior.get(slot).expect("Prior");
                for (index, &(word, width)) in FIELDS.iter().enumerate() {
                    let value = if width == 0 {
                        0x7fc0_4321 + u32::try_from(index).expect("Field")
                    } else {
                        (0x5a5a_5a5a + u32::try_from(index).expect("Field"))
                            & (u32::MAX >> (32 - width))
                    };
                    if width == 0 {
                        raw_float(writer, old.word(word), value);
                    } else {
                        integer(writer, width, old.word(word), value);
                    }
                }
            }
        });
        let result = parse(profile, &bytes, Some(&prior));
        for objective in result.objectives() {
            assert_eq!(objective.status(), 5);
            for (index, &(word, width)) in FIELDS.iter().enumerate() {
                assert_eq!(
                    objective.word(word),
                    if width == 0 {
                        0x7fc0_4321 + u32::try_from(index).expect("Field")
                    } else {
                        (0x5a5a_5a5a + u32::try_from(index).expect("Field"))
                            & (u32::MAX >> (32 - width))
                    }
                );
            }
        }
    }
}

#[test]
fn float_forms_share_signed_zero_raw_nan_and_compact_prior_truncation() {
    for (prior_bits, payload, expected) in [
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
        let prior = seed(PROFILES[1], prior_bits);
        let bytes = encoded(|writer| {
            writer.write_bits(1, 1).expect("Section changed");
            for _ in 0..16 {
                writer.write_bits(0, 3).expect("Status");
                writer.write_bits(1, 1).expect("Changed entry");
                writer.write_bits(1, 1).expect("Changed X");
                writer
                    .write_bits(
                        u32::from(payload[0]),
                        if payload[0] & 3 == 1 { 7 } else { 2 },
                    )
                    .expect("Independent float bits");
                for &byte in &payload[1..] {
                    writer.write_u8(byte).expect("Independent float bytes");
                }
                for _ in 1..6 {
                    writer.write_bits(0, 1).expect("Unchanged field");
                }
            }
        });
        let result = parse(PROFILES[1], &bytes, Some(&prior));
        assert!(
            result
                .objectives()
                .all(|objective| objective.word(W::OriginX) == expected)
        );
        assert!(
            result
                .objectives()
                .all(|objective| objective.word(W::OriginY) == prior_bits)
        );
    }
}

#[test]
fn cached_bits_and_following_bytes_keep_their_independent_cursors() {
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Prior cached field");
        writer.write_bits(1, 1).expect("Section changed");
        for _ in 0..16 {
            writer
                .write_bits(7, 4)
                .expect("Status seven, unchanged words");
        }
        writer.write_u8(0xa7).expect("Following byte");
        writer.write_bits(3, 2).expect("Following cached field");
    });
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    assert_eq!(reader.read_bits(3).expect("Prior field"), 5);
    let result = SnapshotObjectives::parse(&mut reader, PROFILES[1], None).expect("Section");
    assert!(result.objectives().all(|objective| objective.status() == 7));
    assert_eq!(reader.read_u8().expect("Following byte"), 0xa7);
    assert_eq!(reader.read_bits(2).expect("Following bits"), 3);
}
