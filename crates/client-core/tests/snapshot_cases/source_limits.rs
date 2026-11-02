// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, Change, ClientStreamError, EntityStateSource, EntityStreamError, SnapshotBaselines,
    SnapshotError, SnapshotPlan, body, contexts, dirty, input, output,
};

#[test]
fn unmentioned_prior_entities_and_clients_charge_independent_inspection_budgets_before_reads() {
    for context in contexts() {
        let bytes = body(0, true, &[Change::Set(4, 0, 7)], &[Change::Set(5, 0, 511)]);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Prior body");
        let (mut old_entities, mut old_clients) = dirty();
        let old = SnapshotPlan::prepare(
            &mut reader,
            input(context, 10, None),
            output(&mut old_entities, &mut old_clients),
        )
        .expect("Prior")
        .commit_transmitted()
        .expect("Commit");
        for client_limit in [false, true] {
            let mut reader = BodyReader::try_new(&[], 0).expect("No wire needed");
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            let mut selected = input(context, 11, Some(old.as_source()));
            let expected = if client_limit {
                selected.budget.clients.max_previous_clients = 0;
                SnapshotError::Clients(ClientStreamError::PreviousBudget)
            } else {
                selected.budget.entities.max_previous_entities = 0;
                SnapshotError::Entities(EntityStreamError::PreviousBudget)
            };
            assert_eq!(
                SnapshotPlan::prepare(&mut reader, selected, output(&mut entities, &mut clients))
                    .expect_err("Prior inspection budget"),
                expected
            );
            assert_eq!(reader.byte_position(), 0);
            assert_eq!((entities, clients), saved);
        }
    }
}
#[test]
fn even_unused_duplicate_initial_baselines_reject_before_header_and_output_changes() {
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
        let state = seed.entities().entities().next().expect("Seed entity");
        let baseline = [state.clone(), state.clone()];
        let mut selected = input(context, 2, None);
        selected.baselines = SnapshotBaselines::new(
            7,
            context.profile(),
            EntityStateSource::from_states(&baseline),
        );
        let mut reader = BodyReader::try_new(&[], 0).expect("No header needed");
        let (mut entities, mut clients) = dirty();
        let saved = (entities.clone(), clients.clone());
        assert_eq!(
            SnapshotPlan::prepare(&mut reader, selected, output(&mut entities, &mut clients))
                .expect_err("Invalid baseline source"),
            SnapshotError::Entities(EntityStreamError::Source { baseline: true })
        );
        assert_eq!(reader.byte_position(), 0);
        assert_eq!((entities, clients), saved);
    }
}
