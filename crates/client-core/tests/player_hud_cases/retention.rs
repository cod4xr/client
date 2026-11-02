// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    FIELDS, MODERN, SnapshotHud, W, array, encoded, integer, legacy, one_field, parse, raw_float,
    seed,
};

#[test]
fn unchanged_sections_and_omitted_prefix_words_retain_complete_prior_bits() {
    for context in [MODERN, legacy([0.0; 3])] {
        let prior = seed(context, &[1, 1], 0x7fc0_1234);
        assert_eq!(parse(context, 0, &[0], Some(&prior)), prior);
        let bytes = encoded(|writer| {
            writer.write_bits(1, 1).expect("HUD changed");
            array(writer, 1, 0, |_, _, writer| {
                writer.write_bits(0, 1).expect("Unchanged color");
            });
            writer.write_bits(0, 5).expect("No current prefix");
        });
        let next = parse(context, 0, &bytes, Some(&prior));
        assert_eq!(next.archival()[0], prior.archival()[0]);
        for &(word, _) in &FIELDS {
            assert_eq!(next.archival()[1].word(word), 0);
        }
    }
}

#[test]
fn native_tail_clearing_stops_at_first_free_slot_and_keeps_later_inert_words() {
    let prior = seed(MODERN, &[1, 1, 0, 2], 0x7fc0_1234);
    let next = parse(MODERN, 0, &[1, 0], Some(&prior));
    for slot in 0..2 {
        for &(word, _) in &FIELDS {
            assert_eq!(next.archival()[slot].word(word), 0);
        }
    }
    assert_eq!(next.archival()[2], prior.archival()[2]);
    assert_eq!(next.archival()[3], prior.archival()[3]);
    assert_eq!(next.archival()[2].word(W::Kind), 0);
    assert_eq!(next.archival()[2].word(W::FromX), 0x7fc0_1234);
    assert_eq!(next.archival()[3].word(W::Kind), 2);
}

#[test]
fn an_initial_free_slot_retains_its_reference_words_for_a_later_delta() {
    let prior = seed(MODERN, &[0, 1], 0x7fc0_1234);
    let retained = parse(MODERN, 0, &[1, 0], Some(&prior));
    assert_eq!(retained.archival(), prior.archival());
    let next = one_field(MODERN, 0, Some(&retained), W::FromX, |writer| {
        raw_float(writer, 0x7fc0_1234, 1.5_f32.to_bits());
    });
    assert_eq!(next.archival()[0].word(W::FromX), 1.5_f32.to_bits());
    // The next prefix retained slot zero but now clears the following nonfree slot.
    for &(word, _) in &FIELDS {
        assert_eq!(next.archival()[1].word(word), 0);
    }
}

#[test]
fn all_nonfree_remaining_slots_clear_when_both_serialized_counts_become_zero() {
    let prior = seed(MODERN, &[1; 31], 0xdead_beef);
    let next = parse(MODERN, 0, &[1, 0], Some(&prior));
    assert_eq!(next, SnapshotHud::zero(MODERN));
}

#[test]
fn cooked_hud_outlives_messages_and_priors_and_debug_hides_raw_words() {
    let next = {
        let prior = seed(MODERN, &[1], 0xdead_beef);
        let bytes = encoded(|writer| {
            writer.write_bits(1, 1).expect("HUD changed");
            array(writer, 1, 39, |_, index, writer| {
                if FIELDS[index].0 == W::Width {
                    integer(writer, 10, 0, 17);
                } else {
                    writer.write_bits(0, 1).expect("Unchanged");
                }
            });
            writer.write_bits(0, 5).expect("No current prefix");
        });
        parse(MODERN, 0, &bytes, Some(&prior))
    };
    assert_eq!(next.archival()[0].word(W::Value), 0xdead_beef);
    assert_eq!(next.archival()[0].word(W::Width), 17);
    let debug = format!("{next:?} {:?}", next.archival()[0]);
    for private in ["dead", "beef", "3735928559"] {
        assert!(!debug.contains(private));
    }
}
