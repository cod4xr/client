// SPDX-License-Identifier: AGPL-3.0-or-later
use super::support::{full, header, selected};
use crate::snapshots::{
    BodyReader, SnapshotClientSlot, SnapshotEntitySlot, SnapshotMessage, SnapshotPlan, body,
    contexts, output,
};
use cod4xr_client_core::snapshots::history::{
    SnapshotHistory, SnapshotHistoryError, SnapshotHistorySlot,
};

#[test]
fn wrong_world_profile_and_stale_messages_never_replace_committed_history() {
    let [modern, legacy] = contexts();
    let mut entities = [SnapshotEntitySlot::default()];
    let mut clients = [SnapshotClientSlot::default()];
    let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
    let mut history = SnapshotHistory::new(7, modern.profile(), &mut slots).expect("History");
    let mut entity_output = [SnapshotEntitySlot::default()];
    let mut client_output = [SnapshotClientSlot::default()];
    let plan = full(modern, 7, 9, &mut entity_output, &mut client_output);
    let _ = history.prepare(&plan).expect("Initial").commit();
    for (context, epoch, sequence, expected) in [
        (modern, 8, 10, SnapshotHistoryError::Epoch),
        (legacy, 7, 10, SnapshotHistoryError::Profile),
        (modern, 7, 9, SnapshotHistoryError::MessageOrder),
        (modern, 7, 8, SnapshotHistoryError::MessageOrder),
    ] {
        let plan = full(
            context,
            epoch,
            sequence,
            &mut entity_output,
            &mut client_output,
        );
        assert_eq!(
            history.prepare(&plan).expect_err("Binding rejection"),
            expected
        );
        assert_eq!(
            history
                .reference(
                    SnapshotMessage::new(epoch, sequence, 3).expect("Message"),
                    header(context, 0)
                )
                .expect_err("Same header binding"),
            expected
        );
        assert_eq!((history.newest_sequence(), history.len()), (Some(9), 1));
        assert!(history.source(9).is_some());
    }
}
#[test]
fn next_slot_capacities_reject_before_eviction_and_reference_arithmetic_is_checked() {
    let context = contexts()[0];
    assert_eq!(
        SnapshotHistory::new(7, context.profile(), &mut []).expect_err("No retention storage"),
        SnapshotHistoryError::Storage
    );
    let mut entity_output = [SnapshotEntitySlot::default()];
    let mut client_output = [SnapshotClientSlot::default()];
    let plan = full(context, 7, 2, &mut entity_output, &mut client_output);
    for entity_capacity in [true, false] {
        let mut entities = vec![SnapshotEntitySlot::default(); usize::from(!entity_capacity)];
        let mut clients = vec![SnapshotClientSlot::default(); usize::from(entity_capacity)];
        let old_entities = entities.clone();
        let old_clients = clients.clone();
        {
            let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
            let mut history = SnapshotHistory::new(7, context.profile(), &mut slots).expect("Slot");
            let bytes = body(0, true, &[], &[]);
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Empty snapshot");
            let empty = SnapshotPlan::prepare(
                &mut reader,
                selected(context, 7, 1),
                output(&mut [], &mut []),
            )
            .expect("Zero sparse state")
            .commit_transmitted()
            .expect("Transmitted");
            let _ = history
                .prepare(&empty)
                .expect("Retain empty snapshot in tiny slot")
                .commit();
            let expected = if entity_capacity {
                SnapshotHistoryError::EntityCapacity
            } else {
                SnapshotHistoryError::ClientCapacity
            };
            assert_eq!(
                history
                    .prepare(&plan)
                    .expect_err("Separate sparse capacity"),
                expected
            );
            assert_eq!((history.len(), history.newest_sequence()), (1, Some(1)));
            assert!(history.source(1).is_some());
            assert!(history.source(2).is_none());
            assert_eq!(
                history
                    .reference(
                        SnapshotMessage::new(7, 2, 3).expect("Message"),
                        header(context, 2)
                    )
                    .expect_err("Sequence zero cannot be a reference"),
                SnapshotHistoryError::ReferenceSequence
            );
            assert_eq!(
                history
                    .reference(
                        SnapshotMessage::new(7, 2, 3).expect("Message"),
                        header(context, 255)
                    )
                    .expect_err("Checked subtraction"),
                SnapshotHistoryError::ReferenceSequence
            );
        }
        assert_eq!(entities, old_entities);
        assert_eq!(clients, old_clients);
    }
}
#[test]
fn newer_epoch_reset_invalidates_every_reference_and_can_replace_the_profile() {
    let [modern, legacy] = contexts();
    let mut entities = [SnapshotEntitySlot::default()];
    let mut clients = [SnapshotClientSlot::default()];
    let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
    let mut history = SnapshotHistory::new(7, modern.profile(), &mut slots).expect("History");
    let mut entity_output = [SnapshotEntitySlot::default()];
    let mut client_output = [SnapshotClientSlot::default()];
    let plan = full(modern, 7, 9, &mut entity_output, &mut client_output);
    let _ = history.prepare(&plan).expect("Initial").commit();
    for epoch in [6, 7] {
        assert_eq!(
            history.reset(epoch, legacy.profile()),
            Err(SnapshotHistoryError::EpochOrder)
        );
        assert_eq!(
            (
                history.epoch(),
                history.profile(),
                history.newest_sequence()
            ),
            (7, modern.profile(), Some(9))
        );
    }
    history
        .reset(8, legacy.profile())
        .expect("New explicit world");
    assert_eq!(
        (history.epoch(), history.profile(), history.len()),
        (8, legacy.profile(), 0)
    );
    assert!(history.source(9).is_none());
    assert!(history.header(9).is_none());
    assert_eq!(history.newest_sequence(), None);
    let plan = full(legacy, 8, 1, &mut entity_output, &mut client_output);
    assert_eq!(
        history
            .prepare(&plan)
            .expect("Sequence restarted in new epoch")
            .commit()
            .evicted,
        None
    );
    assert_eq!(
        history.source(1).expect("New coherent source").profile(),
        legacy.profile()
    );
}
