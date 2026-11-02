// SPDX-License-Identifier: AGPL-3.0-or-later
use super::support::{full, header, selected};
use crate::snapshots::{
    BodyReader, Change, EntityDecodeContext, SnapshotClientSlot, SnapshotEntitySlot,
    SnapshotMessage, SnapshotPlan, body, contexts, dirty, output, values,
};
use cod4xr_client_core::snapshots::history::{
    SnapshotHistory, SnapshotHistoryError, SnapshotHistorySlot,
};

#[test]
fn retained_complete_states_outlive_source_wire_and_output_then_decode_exact_delta() {
    for context in contexts() {
        let mut entities = vec![SnapshotEntitySlot::default(); 2];
        let mut clients = vec![SnapshotClientSlot::default(); 2];
        let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
        let mut history = SnapshotHistory::new(7, context.profile(), &mut slots).expect("History");
        let expected_player = {
            let mut entity_output = vec![SnapshotEntitySlot::default(); 2];
            let mut client_output = vec![SnapshotClientSlot::default(); 2];
            let plan = full(context, 7, 9, &mut entity_output, &mut client_output);
            let expected = plan.player().clone();
            let summary = history.prepare(&plan).expect("History preflight").commit();
            assert_eq!(
                (summary.sequence, summary.evicted, summary.retained),
                (9, None, 1)
            );
            expected
        };
        assert_eq!(
            history.header(9).expect("Retained header").server_time(),
            0x7654_3210
        );
        let message = SnapshotMessage::new(7, 12, 4).expect("Next containing message");
        let prior = history
            .reference(message, header(context, 3))
            .expect("Exact prior");
        assert_eq!(prior.expect("Delta reference").message().sequence(), 9);
        let bytes = body(3, true, &[Change::Keep(1)], &[Change::Keep(63)]);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Delta bytes");
        let mut entity_output = vec![SnapshotEntitySlot::default(); 2];
        let mut client_output = vec![SnapshotClientSlot::default(); 2];
        let mut current = selected(context, 7, 12);
        current.message = message;
        current.previous = prior;
        let plan = SnapshotPlan::prepare(
            &mut reader,
            current,
            output(&mut entity_output, &mut client_output),
        )
        .expect("History-backed delta")
        .commit_transmitted()
        .expect("Complete state");
        assert_eq!(values(&plan), (vec![(1, 13)], vec![(63, 511)]));
        assert_eq!(plan.player(), &expected_player);
        // Returned plans have no prior-history borrow, allowing the next retention.
        assert_eq!(
            history
                .prepare(&plan)
                .expect("Replace one-slot history")
                .commit()
                .evicted,
            Some(9)
        );
        assert!(history.source(9).is_none());
        assert_eq!(history.source(12).expect("Replacement").message(), message);
    }
}
#[test]
fn configurable_retention_exceeds_native_ring_and_preserves_all_wire_distances() {
    let context = EntityDecodeContext::Cod4x21;
    let mut entities = vec![vec![SnapshotEntitySlot::default(); 1]; 40];
    let mut clients = vec![vec![SnapshotClientSlot::default(); 1]; 40];
    let mut slots: Vec<_> = entities
        .iter_mut()
        .zip(&mut clients)
        .map(|(entities, clients)| SnapshotHistorySlot::new(entities, clients))
        .collect();
    let mut history =
        SnapshotHistory::new(7, context.profile(), &mut slots).expect("Forty host slots");
    let mut entity_output = [SnapshotEntitySlot::default()];
    let mut client_output = [SnapshotClientSlot::default()];
    for index in 0..40 {
        let sequence = index * 3 + 1;
        let plan = full(context, 7, sequence, &mut entity_output, &mut client_output);
        let summary = history
            .prepare(&plan)
            .expect("Retention preflight")
            .commit();
        assert_eq!(summary.evicted, None);
    }
    assert_eq!((history.len(), history.capacity()), (40, 40));
    for sequence in (1..119).step_by(3) {
        assert_eq!(
            history
                .source(sequence)
                .expect("No native ring alias")
                .message()
                .sequence(),
            sequence
        );
    }
    assert_eq!(
        history
            .reference(
                SnapshotMessage::new(7, 256, 4).expect("Distance 255"),
                header(context, 255)
            )
            .expect("Full byte distance")
            .expect("Exact oldest")
            .message()
            .sequence(),
        1
    );
    assert_eq!(
        history
            .reference(
                SnapshotMessage::new(7, 257, 4).expect("Missing sequence"),
                header(context, 255)
            )
            .expect_err("Never substitute nearby retained data"),
        SnapshotHistoryError::MissingReference
    );
    assert!(
        history
            .reference(
                SnapshotMessage::new(7, 256, 4).expect("Full message"),
                header(context, 0)
            )
            .expect("Full snapshot")
            .is_none()
    );
    let plan = full(context, 7, 1000, &mut entity_output, &mut client_output);
    assert_eq!(
        history
            .prepare(&plan)
            .expect("Insertion gap uses one slot")
            .commit()
            .evicted,
        Some(1)
    );
    assert_eq!(history.len(), 40);
    assert!(history.source(1).is_none());
    assert!(history.source(4).is_some());
    assert!(history.source(0).is_none());
    assert!(history.header(999).is_none());
}
#[test]
fn dropping_prepared_replacement_preserves_original_state_metadata_and_unused_tails() {
    let context = EntityDecodeContext::Cod4x21;
    let (mut entities, mut clients) = dirty();
    let before_entities = entities.clone();
    let before_clients = clients.clone();
    let mut entity_output = vec![SnapshotEntitySlot::default(); 2];
    let mut client_output = vec![SnapshotClientSlot::default(); 2];
    {
        let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
        let mut history = SnapshotHistory::new(7, context.profile(), &mut slots).expect("History");
        let plan = full(context, 7, 1, &mut entity_output, &mut client_output);
        drop(history.prepare(&plan).expect("Cancelled initial retention"));
        assert!(history.is_empty());
        assert_eq!(history.newest_sequence(), None);
        let _ = history
            .prepare(&plan)
            .expect("Commit initial state")
            .commit();
        let mut other_entities = [SnapshotEntitySlot::default()];
        let mut other_clients = [SnapshotClientSlot::default()];
        let other = full(context, 7, 2, &mut other_entities, &mut other_clients);
        drop(
            history
                .prepare(&other)
                .expect("Cancel replacing occupied slot"),
        );
        assert_eq!(history.newest_sequence(), Some(1));
        assert!(history.source(1).is_some());
        assert!(history.source(2).is_none());
        assert!(!format!("{history:?}").contains("1985229328"));
    }
    assert_eq!(entities[0], entity_output[0]);
    assert_eq!(clients[0], client_output[0]);
    assert_eq!(entities[1..], before_entities[1..]);
    assert_eq!(clients[1..], before_clients[1..]);
}
