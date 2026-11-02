// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    ArchiveId, BUDGET, MountCandidate, MountIndex, MountIndexBudget, MountScratchSlot, dirty_slots,
    keys, metadata,
};
use cod4xr_content::mount_index::MountIndexError;

#[test]
fn all_input_retention_and_storage_bounds_preserve_both_final_outputs() {
    let mut storage = vec![vec![0; 2]; 3];
    let keys = keys(&[b"aa", b"b", b"aa"], &mut storage);
    let inputs: Vec<_> = keys
        .into_iter()
        .map(|key| MountCandidate::new(key, ArchiveId::new(1), metadata(1)))
        .collect();
    let exact = MountIndexBudget {
        max_candidates: 3,
        max_input_key_bytes: 5,
        max_files: 2,
        max_key_bytes: 3,
    };
    for (budget, expected) in [
        (
            MountIndexBudget {
                max_candidates: 2,
                ..exact
            },
            MountIndexError::CandidateBudget,
        ),
        (
            MountIndexBudget {
                max_input_key_bytes: 4,
                ..exact
            },
            MountIndexError::InputKeyBudget,
        ),
        (
            MountIndexBudget {
                max_files: 1,
                ..exact
            },
            MountIndexError::FileBudget,
        ),
        (
            MountIndexBudget {
                max_key_bytes: 2,
                ..exact
            },
            MountIndexError::KeyBudget,
        ),
    ] {
        let mut text = [0xa5; 3];
        let mut slots = dirty_slots(2);
        let before = slots.clone();
        assert_eq!(
            MountIndex::build(
                &inputs,
                budget,
                &mut [MountScratchSlot::default(); 3],
                &mut text,
                &mut slots
            )
            .err(),
            Some(expected)
        );
        assert_eq!(text, [0xa5; 3]);
        assert_eq!(slots, before);
    }
    for (scratch_count, slot_count, text_count, expected) in [
        (2, 2, 3, MountIndexError::ScratchCapacity),
        (3, 1, 3, MountIndexError::SlotCapacity),
        (3, 2, 2, MountIndexError::OutputCapacity),
    ] {
        let mut text = vec![0xa5; text_count];
        let before_text = text.clone();
        let mut slots = dirty_slots(slot_count);
        let before_slots = slots.clone();
        assert_eq!(
            MountIndex::build(
                &inputs,
                exact,
                &mut vec![MountScratchSlot::default(); scratch_count],
                &mut text,
                &mut slots
            )
            .err(),
            Some(expected)
        );
        assert_eq!(text, before_text);
        assert_eq!(slots, before_slots);
    }
    let mut text = [0; 3];
    let mut slots = dirty_slots(2);
    let index = MountIndex::build(
        &inputs,
        exact,
        &mut [MountScratchSlot::default(); 3],
        &mut text,
        &mut slots,
    )
    .expect("Exact budgets/capacities");
    assert_eq!(index.stats().files, 2);
    assert_eq!(index.stats().key_bytes, 3);
}

#[test]
fn every_short_key_and_descriptor_capacity_rolls_back_even_after_earlier_winners() {
    let mut storage = vec![vec![0; 3]; 3];
    let keys = keys(&[b"a", b"bb", b"ccc"], &mut storage);
    let inputs: Vec<_> = keys
        .into_iter()
        .map(|key| MountCandidate::new(key, ArchiveId::new(1), metadata(1)))
        .collect();
    for count in 0..6 {
        let mut text = vec![0xa5; count];
        let before = text.clone();
        let mut slots = dirty_slots(3);
        let before_slots = slots.clone();
        assert_eq!(
            MountIndex::build(
                &inputs,
                BUDGET,
                &mut [MountScratchSlot::default(); 3],
                &mut text,
                &mut slots
            )
            .err(),
            Some(MountIndexError::OutputCapacity)
        );
        assert_eq!(text, before);
        assert_eq!(slots, before_slots);
    }
    for count in 0..3 {
        let mut text = [0xa5; 6];
        let mut slots = dirty_slots(count);
        let before = slots.clone();
        assert_eq!(
            MountIndex::build(
                &inputs,
                BUDGET,
                &mut [MountScratchSlot::default(); 3],
                &mut text,
                &mut slots
            )
            .err(),
            Some(MountIndexError::SlotCapacity)
        );
        assert_eq!(text, [0xa5; 6]);
        assert_eq!(slots, before);
    }
}
