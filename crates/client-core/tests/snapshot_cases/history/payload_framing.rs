// SPDX-License-Identifier: AGPL-3.0-or-later
use super::payload_support::{Attempt, outgoing, request, seed};
use crate::snapshots::{
    PlayerStateError, SnapshotClientSlot, SnapshotEntitySlot, SnapshotError, SnapshotMessage,
    contexts,
    mixed_support::{Finals, carrier},
};
use cod4xr_client_core::{
    commands::LEGACY_COMMAND_SLOT_BYTES,
    server_payload::MixedSnapshotAcknowledgementError,
    service_controls::ServiceControlError,
    snapshots::{
        history::{SnapshotHistory, SnapshotHistorySlot},
        mixed_services::MixedSnapshotError,
        services::SnapshotPayloadError,
    },
};

#[test]
fn actual_binding_and_prefix_validation_precede_history_resolution_without_publishing() {
    let [context, other] = contexts();
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut outgoing = outgoing(&mut raw);
    let mut entities = [SnapshotEntitySlot::default()];
    let mut clients = [SnapshotClientSlot::default()];
    let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
    let history = SnapshotHistory::new(8, context.profile(), &mut slots).expect("Wrong world");
    let mut finals = Finals::new();
    let saved = finals.clone();
    let mut attempt = Attempt::new(&carrier(context.profile(), 0, true), context);
    for profile_mismatch in [true, false] {
        let mut current = request(if profile_mismatch { other } else { context }, &history);
        current.services.max_records = 0;
        if !profile_mismatch {
            current.message = SnapshotMessage::new(7, 8, 3).expect("Wrong message");
        }
        let expected = if profile_mismatch {
            SnapshotPayloadError::Profile
        } else {
            SnapshotPayloadError::MessageSequence
        };
        assert_eq!(
            attempt
                .prepare(&mut outgoing, current, &mut finals)
                .expect_err("Binding before prefix budget/history"),
            MixedSnapshotAcknowledgementError::Payload(MixedSnapshotError::Binding(expected))
        );
        assert_eq!(finals, saved);
    }
    for (source, expected) in [
        (
            b"\xfe".as_slice(),
            MixedSnapshotError::Controls(ServiceControlError::UnsupportedService(0xfe)),
        ),
        (b"\x07".as_slice(), MixedSnapshotError::MissingSnapshot),
    ] {
        let mut attempt = Attempt::new(source, context);
        assert_eq!(
            attempt
                .prepare(&mut outgoing, request(context, &history), &mut finals)
                .expect_err("Prefix before wrong-world lookup"),
            MixedSnapshotAcknowledgementError::Payload(expected)
        );
        assert_eq!(finals, saved);
    }
    assert_eq!(outgoing.acknowledged_sequence(), 0);
}
#[test]
fn late_wire_and_prediction_failure_after_exact_history_selection_preserves_all_outputs() {
    for context in contexts() {
        for prediction_failure in [false, true] {
            let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
            let mut outgoing = outgoing(&mut raw);
            let mut entities = [SnapshotEntitySlot::default()];
            let mut clients = [SnapshotClientSlot::default()];
            let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
            let mut history =
                SnapshotHistory::new(7, context.profile(), &mut slots).expect("History");
            seed(&mut history, context, 5);
            let mut source = carrier(context.profile(), 2, !prediction_failure);
            if !prediction_failure {
                *source.last_mut().expect("EOF") = 0xfe;
            }
            let mut attempt = Attempt::new(&source, context);
            let mut finals = Finals::new();
            let saved = finals.clone();
            let expected = if prediction_failure {
                MixedSnapshotError::Snapshot(SnapshotError::Player(
                    PlayerStateError::PredictionRequired,
                ))
            } else {
                MixedSnapshotError::Controls(ServiceControlError::UnsupportedService(0xfe))
            };
            assert_eq!(
                attempt
                    .prepare(&mut outgoing, request(context, &history), &mut finals)
                    .expect_err("Late full-carrier failure"),
                MixedSnapshotAcknowledgementError::Payload(expected)
            );
            assert_eq!(finals, saved);
            assert_eq!(
                (outgoing.acknowledged_sequence(), outgoing.pending_count()),
                (0, 1)
            );
            assert_eq!(history.newest_sequence(), Some(5));
        }
    }
}
#[test]
fn prefix_and_tail_use_the_original_independent_control_budgets() {
    let context = contexts()[0];
    for mode in 0..5 {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let mut outgoing = outgoing(&mut raw);
        let mut entities = [SnapshotEntitySlot::default()];
        let mut clients = [SnapshotClientSlot::default()];
        let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
        let mut history = SnapshotHistory::new(7, context.profile(), &mut slots).expect("History");
        seed(&mut history, context, 5);
        let mut attempt = Attempt::new(&carrier(context.profile(), 2, true), context);
        let mut finals = Finals::new();
        let mut current = request(context, &history);
        let expected = match mode {
            0 => {
                current.services.max_records = 1;
                ServiceControlError::RecordBudget
            }
            1 => {
                current.services.max_client_updates = 0;
                ServiceControlError::ClientDataBudget
            }
            2 => {
                finals.text.truncate(0);
                ServiceControlError::OutputCapacity
            }
            3 => {
                finals.identities.truncate(0);
                ServiceControlError::ClientDataCapacity
            }
            _ => {
                current.services.max_client_updates = 1;
                ServiceControlError::ClientDataBudget
            }
        };
        let saved = finals.clone();
        assert_eq!(
            attempt
                .prepare(&mut outgoing, current, &mut finals)
                .expect_err("Independent prefix/tail policy"),
            MixedSnapshotAcknowledgementError::Payload(MixedSnapshotError::Controls(expected))
        );
        assert_eq!(finals, saved);
        assert_eq!(outgoing.acknowledged_sequence(), 0);
        assert_eq!(history.newest_sequence(), Some(5));
    }
}
