// SPDX-License-Identifier: AGPL-3.0-or-later
use super::payload_support::{Attempt, outgoing, request, seed};
use crate::snapshots::{
    SnapshotClientSlot, SnapshotEntitySlot, contexts,
    mixed_support::{Finals, carrier},
    values,
};
use cod4xr_client_core::{
    commands::LEGACY_COMMAND_SLOT_BYTES,
    snapshots::history::{SnapshotHistory, SnapshotHistorySlot},
};

#[test]
fn actual_header_selects_exact_prior_in_both_profiles_and_releases_history_borrow_before_retention()
{
    for context in contexts() {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let mut outgoing = outgoing(&mut raw);
        let mut entities = [SnapshotEntitySlot::default()];
        let mut clients = [SnapshotClientSlot::default()];
        let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
        let mut history = SnapshotHistory::new(7, context.profile(), &mut slots).expect("History");
        seed(&mut history, context, 5);
        let mut finals = Finals::new();
        let pending = {
            let source = carrier(context.profile(), 2, true);
            let mut attempt = Attempt::new(&source, context);
            attempt
                .prepare(&mut outgoing, request(context, &history), &mut finals)
                .expect("Automatic exact reference")
        };
        assert_eq!(
            values(pending.plan().snapshot()),
            (vec![(1, 13)], vec![(63, 511)])
        );
        assert_eq!(
            pending
                .plan()
                .snapshot()
                .message()
                .server_command_sequence(),
            4
        );
        assert!(!format!("{pending:?}").contains("Private"));
        // This exclusive child proves that neither decode nor private inspection
        // leaves a prior-history borrow in the returned prepared mixed result.
        let retained = history
            .prepare(pending.plan().snapshot())
            .expect("All history resources")
            .commit();
        assert_eq!(retained.evicted, Some(5));
        let plan = pending.commit();
        assert_eq!(outgoing.acknowledged_sequence(), 1);
        assert_eq!(
            history.source(7).expect("New complete source").message(),
            plan.snapshot().message()
        );
        assert!(history.source(5).is_none());
    }
}
#[test]
fn full_header_uses_no_prior_even_with_retained_history_and_successful_drop_keeps_ack_pending() {
    for context in contexts() {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let mut outgoing = outgoing(&mut raw);
        let mut entities = [SnapshotEntitySlot::default()];
        let mut clients = [SnapshotClientSlot::default()];
        let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
        let mut history = SnapshotHistory::new(7, context.profile(), &mut slots).expect("History");
        seed(&mut history, context, 5);
        let mut attempt = Attempt::new(&carrier(context.profile(), 0, true), context);
        let mut finals = Finals::new();
        let pending = attempt
            .prepare(&mut outgoing, request(context, &history), &mut finals)
            .expect("Full header ignores old prior");
        assert_eq!(
            values(pending.plan().snapshot()),
            (vec![(1, 13)], vec![(63, 511)])
        );
        drop(pending);
        assert_eq!(
            (outgoing.acknowledged_sequence(), outgoing.pending_count()),
            (0, 1)
        );
        assert_eq!(history.newest_sequence(), Some(5));
        let plan = attempt
            .prepare(&mut outgoing, request(context, &history), &mut finals)
            .expect("Same original key retry")
            .commit();
        assert_eq!(plan.controls().client_command_acknowledge(), 1);
        assert_eq!(outgoing.acknowledged_sequence(), 1);
        assert_eq!(history.newest_sequence(), Some(5));
    }
}
