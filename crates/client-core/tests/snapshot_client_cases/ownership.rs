// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BUDGET, BodyReader, PROFILES, SnapshotClientPlan, SnapshotClientSlot, Update, dirty, fixture,
    source, state, values,
};

#[test]
fn cooked_result_outlives_input_previous_states_and_decoder_without_exposing_tail() {
    let mut output = dirty(5);
    let saved = output.clone();
    let plan = {
        let previous = [state(1, 101), state(63, 163)];
        let bytes = fixture(&[Update::Model(5, 0, 511)], false, false);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        SnapshotClientPlan::parse(
            &mut reader,
            PROFILES[1],
            source(&previous),
            BUDGET,
            &mut output,
        )
        .expect("Owned cooked result")
    };
    assert_eq!(values(&plan), [(1, 101), (5, 511), (63, 163)]);
    assert_eq!(plan.profile(), PROFILES[1]);
    assert_eq!(plan.as_source().len(), 3);
    assert!(!plan.as_source().is_empty());
    assert_eq!(&output[3..], &saved[3..]);
}

#[test]
fn cooked_source_outlives_its_parent_plan_and_reuses_cells_without_repacking() {
    let mut first_output = dirty(4);
    let prior = {
        let previous = [state(7, 341), state(63, 163)];
        let bytes = fixture(&[], false, false);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let plan = SnapshotClientPlan::parse(
            &mut reader,
            PROFILES[1],
            source(&previous),
            BUDGET,
            &mut first_output,
        )
        .expect("First plan");
        plan.as_source()
    };
    for _ in 0..3 {
        let bytes = fixture(&[Update::Model(7, 341, 17)], false, false);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Next body");
        let mut output = dirty(4);
        let next = SnapshotClientPlan::parse(&mut reader, PROFILES[1], prior, BUDGET, &mut output)
            .expect("Reused cooked prior");
        assert_eq!(values(&next), [(7, 17), (63, 163)]);
    }
}

#[test]
fn many_owned_results_can_retain_more_states_than_the_native_global_client_ring() {
    let updates: Vec<_> = (0..64)
        .map(|number| Update::Model(number, 0, u16::try_from(number + 1).expect("Value")))
        .collect();
    let bytes = fixture(&updates, false, false);
    let mut storage: Vec<_> = (0..40)
        .map(|_| vec![SnapshotClientSlot::default(); 64])
        .collect();
    let plans: Vec<_> = storage
        .iter_mut()
        .map(|slots| {
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
            SnapshotClientPlan::parse(&mut reader, PROFILES[1], source(&[]), BUDGET, slots)
                .expect("Independent result")
        })
        .collect();
    assert_eq!(
        plans
            .iter()
            .map(|plan| plan.clients().count())
            .sum::<usize>(),
        2560
    );
    assert!(
        plans
            .iter()
            .all(|plan| values(plan).last() == Some(&(63, 64)))
    );
    // These are independently retained codec results, not admitted snapshot history.
}
