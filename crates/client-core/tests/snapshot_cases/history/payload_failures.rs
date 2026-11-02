// SPDX-License-Identifier: AGPL-3.0-or-later
use super::payload_support::{Attempt, outgoing, request, seed};
use crate::snapshots::{
    SnapshotClientSlot, SnapshotEntitySlot, contexts,
    mixed_support::{Finals, carrier},
};
use cod4xr_client_core::{
    commands::LEGACY_COMMAND_SLOT_BYTES,
    server_payload::MixedSnapshotAcknowledgementError,
    snapshots::history::{SnapshotHistory, SnapshotHistoryError, SnapshotHistorySlot},
};

#[test]
fn exact_history_binding_and_distance_failures_preserve_all_final_buffers_and_original_key() {
    for [context, other] in [contexts(), [contexts()[1], contexts()[0]]] {
        for (owner_context, epoch, sequence, distance, expected) in [
            (context, 7, 5, 1, SnapshotHistoryError::MissingReference),
            (context, 7, 5, 7, SnapshotHistoryError::ReferenceSequence),
            (context, 7, 5, 255, SnapshotHistoryError::ReferenceSequence),
            (context, 8, 5, 0, SnapshotHistoryError::Epoch),
            (other, 7, 5, 0, SnapshotHistoryError::Profile),
            (context, 7, 7, 0, SnapshotHistoryError::MessageOrder),
        ] {
            let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
            let mut outgoing = outgoing(&mut raw);
            let mut entities = [SnapshotEntitySlot::default()];
            let mut clients = [SnapshotClientSlot::default()];
            let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
            let mut history =
                SnapshotHistory::new(epoch, owner_context.profile(), &mut slots).expect("History");
            seed(&mut history, owner_context, sequence);
            let mut attempt = Attempt::new(&carrier(context.profile(), distance, true), context);
            let mut finals = Finals::new();
            let saved = finals.clone();
            let error = attempt
                .prepare(&mut outgoing, request(context, &history), &mut finals)
                .expect_err("Exact history rejection");
            assert_eq!(error, MixedSnapshotAcknowledgementError::History(expected));
            assert_eq!(finals, saved);
            assert_eq!(
                (outgoing.acknowledged_sequence(), outgoing.pending_count()),
                (0, 1)
            );
            assert_eq!(
                outgoing.command(1).expect("Original key").as_bytes(),
                b"PrivateKey"
            );
            assert_eq!(history.newest_sequence(), Some(sequence));
            assert!(history.source(sequence).is_some());
            assert!(!format!("{error:?} {error}").contains("Private"));
        }
    }
}
#[test]
fn evicted_reference_never_falls_back_to_newest_and_full_packet_can_retry_with_original_key() {
    for context in contexts() {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let mut outgoing = outgoing(&mut raw);
        let mut entities = [SnapshotEntitySlot::default()];
        let mut clients = [SnapshotClientSlot::default()];
        let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
        let mut history =
            SnapshotHistory::new(7, context.profile(), &mut slots).expect("One-slot history");
        seed(&mut history, context, 5);
        seed(&mut history, context, 6);
        assert!(history.source(5).is_none());
        let mut attempt = Attempt::new(&carrier(context.profile(), 2, true), context);
        let mut finals = Finals::new();
        let saved = finals.clone();
        assert_eq!(
            attempt
                .prepare(&mut outgoing, request(context, &history), &mut finals)
                .expect_err("Evicted five cannot become six"),
            MixedSnapshotAcknowledgementError::History(SnapshotHistoryError::MissingReference)
        );
        assert_eq!(finals, saved);
        assert_eq!(outgoing.acknowledged_sequence(), 0);
        let mut full = Attempt::new(&carrier(context.profile(), 0, true), context);
        let pending = full
            .prepare(&mut outgoing, request(context, &history), &mut finals)
            .expect("Full packet needs no evicted source");
        let _ = pending.commit();
        assert_eq!(outgoing.acknowledged_sequence(), 1);
        assert_eq!(history.newest_sequence(), Some(6));
    }
}
