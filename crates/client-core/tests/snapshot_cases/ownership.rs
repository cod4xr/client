// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, Change, EntityStateSource, EntityStreamError, SnapshotBaselines, SnapshotError,
    SnapshotPlan, body, contexts, dirty, input, output, values,
};
use std::error::Error;

#[test]
fn whole_result_outlives_prior_plan_and_input_and_keeps_all_cooked_parts() {
    for context in contexts() {
        let (mut entities, mut clients) = dirty();
        let next = {
            let full = body(
                0,
                true,
                &[Change::Set(1, 0, 13)],
                &[Change::Set(63, 0, 511)],
            );
            let mut reader = BodyReader::try_new(&full, full.len()).expect("Prior body");
            let (mut old_entities, mut old_clients) = dirty();
            let old = SnapshotPlan::prepare(
                &mut reader,
                input(context, 10, None),
                output(&mut old_entities, &mut old_clients),
            )
            .expect("Prior")
            .commit_transmitted()
            .expect("Prior commit");
            let delta = body(1, true, &[], &[]);
            let mut reader = BodyReader::try_new(&delta, delta.len()).expect("Delta body");
            SnapshotPlan::prepare(
                &mut reader,
                input(context, 11, Some(old.as_source())),
                output(&mut entities, &mut clients),
            )
            .expect("Delta")
            .commit_transmitted()
            .expect("Commit")
        };
        assert_eq!(values(&next), (vec![(1, 13)], vec![(63, 511)]));
    }
}
#[test]
fn host_funded_history_retains_forty_plans_and_can_reuse_the_oldest_at_distance_255() {
    for context in contexts() {
        let mut storage: Vec<_> = (0..40).map(|_| dirty()).collect();
        let history: Vec<_> = storage
            .iter_mut()
            .enumerate()
            .map(|(index, (entities, clients))| {
                let bytes = body(
                    0,
                    true,
                    &[Change::Set(1, 0, 13)],
                    &[Change::Set(63, 0, 511)],
                );
                let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
                SnapshotPlan::prepare(
                    &mut reader,
                    input(context, u32::try_from(index + 1).expect("Sequence"), None),
                    output(entities, clients),
                )
                .expect("Snapshot")
                .commit_transmitted()
                .expect("Commit")
            })
            .collect();
        let bytes = body(255, true, &[], &[]);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Delta body");
        let (mut entities, mut clients) = dirty();
        let next = SnapshotPlan::prepare(
            &mut reader,
            input(context, 256, Some(history[0].as_source())),
            output(&mut entities, &mut clients),
        )
        .expect("Oldest cooked reference")
        .commit_transmitted()
        .expect("Commit");
        assert_eq!(values(&next), values(&history[0]));
        assert!(history.iter().all(|plan| values(plan) == values(&next)));
    }
}
#[test]
fn baselines_are_used_for_new_entities_and_all_unused_sources_still_charge_policy() {
    for context in contexts() {
        let bytes = body(0, true, &[Change::Set(4, 0, 7)], &[]);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Seed body");
        let (mut old_entities, mut old_clients) = dirty();
        let seed = SnapshotPlan::prepare(
            &mut reader,
            input(context, 1, None),
            output(&mut old_entities, &mut old_clients),
        )
        .expect("Seed")
        .commit_transmitted()
        .expect("Commit");
        let baseline: Vec<_> = seed.entities().entities().cloned().collect();
        let bytes = body(0, true, &[Change::Set(4, 7, 9)], &[]);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let (mut entities, mut clients) = dirty();
        let mut selected = input(context, 2, None);
        selected.baselines = SnapshotBaselines::new(
            7,
            context.profile(),
            EntityStateSource::from_states(&baseline),
        );
        let next =
            SnapshotPlan::prepare(&mut reader, selected, output(&mut entities, &mut clients))
                .expect("Initial baseline reference")
                .commit_transmitted()
                .expect("Commit");
        assert_eq!(values(&next).0, [(4, 9)]);
        assert_eq!(next.entities().stats().baseline_references, 1);
        let mut reader = BodyReader::try_new(&[], 0).expect("No wire needed");
        let (mut entities, mut clients) = dirty();
        let saved = (entities.clone(), clients.clone());
        selected.budget.entities.max_baselines = 0;
        assert_eq!(
            SnapshotPlan::prepare(&mut reader, selected, output(&mut entities, &mut clients))
                .expect_err("Unused source budget"),
            SnapshotError::Entities(EntityStreamError::BaselineBudget)
        );
        assert_eq!((entities, clients), saved);
    }
}
#[test]
fn whole_snapshot_debug_and_error_chains_keep_raw_words_and_labels_private() {
    for context in contexts() {
        let bytes = body(
            0,
            true,
            &[Change::Set(1, 0, 13)],
            &[Change::Set(63, 0, 511)],
        );
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let (mut entities, mut clients) = dirty();
        let prepared = SnapshotPlan::prepare(
            &mut reader,
            input(context, 1, None),
            output(&mut entities, &mut clients),
        )
        .expect("Snapshot");
        let pending = format!("{prepared:?}");
        let plan = prepared.commit_transmitted().expect("Commit");
        let debug = format!("{pending} {plan:?} {:?}", plan.as_source());
        for secret in ["1985229328", "4294967294", "76543210", "511"] {
            assert!(!debug.contains(secret), "{debug}");
        }
        let mut reader = BodyReader::try_new(&[], 0).expect("Empty body");
        let mut entities = [];
        let mut clients = [];
        let error = SnapshotPlan::prepare(
            &mut reader,
            input(context, 1, None),
            output(&mut entities, &mut clients),
        )
        .expect_err("Truncation");
        assert!(error.source().is_some());
        assert!(!error.to_string().contains("76543210"));
    }
}
