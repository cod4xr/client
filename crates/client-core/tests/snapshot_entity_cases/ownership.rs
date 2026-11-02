// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn several_owned_frames_can_retain_more_total_entities_than_the_native_global_ring() {
    let previous: Vec<_> = (0..512).map(|number| state(number, 1)).collect();
    let budget = EntityStreamBudget {
        max_wire_records: 1,
        max_previous_entities: 512,
        max_baselines: 0,
        max_entities: 512,
    };
    let mut frames = vec![vec![SnapshotEntitySlot::default(); 512]; 6];
    let plans: Vec<_> = frames
        .iter_mut()
        .map(|output| {
            let mut reader =
                BodyReader::try_new(&[0xfe, 0x0f], 2).expect("Independent entity terminator");
            SnapshotEntityPlan::parse(
                &mut reader,
                EntityDecodeContext::Cod4x21,
                0,
                sources(&previous, &[]),
                budget,
                output,
            )
            .expect("Independent caller-owned frame")
        })
        .collect();
    assert_eq!(
        plans
            .iter()
            .map(|plan| plan.entities().count())
            .sum::<usize>(),
        3072
    );
    assert!(plans.iter().all(|plan| {
        plan.entities()
            .last()
            .expect("Last retained entity")
            .number()
            == 511
    }));
}

#[test]
fn cooked_plan_outlives_wire_and_both_reference_sources_and_keeps_dirty_tail_private() {
    let mut output = dirty(8);
    let tail = output[3..].to_vec();
    let plan = {
        let previous = [state(1, 1), state(9, 9)];
        let baselines = [state(4, 4)];
        let input = fixture(&[Update::Retain(4)], false, false);
        let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
        SnapshotEntityPlan::parse(
            &mut reader,
            EntityDecodeContext::Cod4x21,
            0,
            sources(&previous, &baselines),
            BUDGET,
            &mut output,
        )
        .expect("Cooked only")
    };
    assert_eq!(values(&plan), [(1, 1), (4, 4), (9, 9)]);
    assert!(!format!("{plan:?}").contains("1065353216"));
    assert_eq!(&output[3..], tail);
}

#[test]
fn source_view_outlives_parent_plan_and_supports_repeated_merges_without_repacking_states() {
    let mut first_output = dirty(8);
    let source = {
        let input = fixture(
            &[Update::Kind(2, 0, 2), Update::Kind(4, 0, 4)],
            false,
            false,
        );
        let mut reader = BodyReader::try_new(&input, input.len()).expect("First");
        let plan = SnapshotEntityPlan::parse(
            &mut reader,
            EntityDecodeContext::Cod4x21,
            0,
            sources(&[], &[]),
            BUDGET,
            &mut first_output,
        )
        .expect("First plan");
        plan.as_source()
    };
    assert_eq!(source.len(), 2);
    assert!(!source.is_empty());
    let input = fixture(&[Update::Kind(2, 2, 12), Update::Remove(4)], false, false);
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Next");
    let mut output = dirty(8);
    let plan = SnapshotEntityPlan::parse(
        &mut reader,
        EntityDecodeContext::Cod4x21,
        0,
        EntityStreamSources {
            previous: source,
            baselines: EntityStateSource::from_states(&[]),
        },
        BUDGET,
        &mut output,
    )
    .expect("Cooked source reuse");
    assert_eq!(values(&plan), [(2, 12)]);
    let mut empty = BodyReader::try_new(&[0xfe, 0x0f], 2).expect("Next terminator");
    let mut final_output = dirty(8);
    let next = SnapshotEntityPlan::parse(
        &mut empty,
        EntityDecodeContext::Cod4x21,
        0,
        EntityStreamSources {
            previous: plan.as_source(),
            baselines: EntityStateSource::from_states(&[]),
        },
        BUDGET,
        &mut final_output,
    )
    .expect("Retain new state");
    assert_eq!(values(&next), [(2, 12)]);
}

#[test]
fn caller_funded_large_sources_have_no_native_command_window_or_ring_allocation_limit() {
    let previous: Vec<_> = (0..512).map(|number| state(number, 1)).collect();
    let budget = EntityStreamBudget {
        max_wire_records: 1,
        max_previous_entities: 512,
        max_baselines: 0,
        max_entities: 512,
    };
    let mut output = vec![SnapshotEntitySlot::default(); 513];
    let mut reader = BodyReader::try_new(&[0xfe, 0x0f], 2).expect("Retain all");
    let plan = SnapshotEntityPlan::parse(
        &mut reader,
        EntityDecodeContext::Cod4x21,
        0,
        sources(&previous, &[]),
        budget,
        &mut output,
    )
    .expect("512 owned retained entities");
    assert_eq!(plan.entities().count(), 512);
    assert_eq!(
        (plan.stats().unchanged_entities, plan.stats().wire_records),
        (512, 1)
    );
    assert_eq!(plan.entities().last().expect("Last entity").number(), 511);
    assert_eq!(output[512], SnapshotEntitySlot::default());
}
