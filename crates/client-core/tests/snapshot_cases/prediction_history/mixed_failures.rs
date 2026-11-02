// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    PredictionHistory, PredictionHistoryError as Error, PredictionSlot, SnapshotError, contexts,
    retain,
};
use crate::snapshots::{
    SnapshotClientSlot, SnapshotEntitySlot,
    history::payload_support::{Attempt, outgoing, request, seed},
    mixed_support::{Finals, carrier},
};
use cod4xr_client_core::{
    commands::LEGACY_COMMAND_SLOT_BYTES,
    server_payload::{MixedSnapshotAcknowledgementError, MixedSnapshotHistoryRequest},
    service_controls::ServiceControlError,
    snapshots::{
        history::{SnapshotHistory, SnapshotHistorySlot},
        mixed_services::MixedSnapshotError,
        services::SnapshotPrediction,
    },
};

#[test]
fn mixed_cache_binding_missing_time_and_late_eof_failure_preserve_all_four_outputs_and_ack() {
    let contexts = contexts();
    for (index, &context) in contexts.iter().enumerate() {
        for (epoch, profile, late, expected) in [
            (
                8,
                context.profile(),
                false,
                MixedSnapshotError::Snapshot(SnapshotError::PredictionHistory(Error::Epoch)),
            ),
            (
                7,
                contexts[1 - index].profile(),
                false,
                MixedSnapshotError::Snapshot(SnapshotError::PredictionHistory(Error::Profile)),
            ),
            (
                7,
                context.profile(),
                false,
                MixedSnapshotError::Snapshot(SnapshotError::PredictionHistory(Error::Missing)),
            ),
            (
                8,
                context.profile(),
                true,
                MixedSnapshotError::Controls(ServiceControlError::UnsupportedService(0xfe)),
            ),
        ] {
            let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
            let mut outgoing = outgoing(&mut raw);
            let mut entities = [SnapshotEntitySlot::default()];
            let mut clients = [SnapshotClientSlot::default()];
            let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
            let mut snapshots =
                SnapshotHistory::new(7, context.profile(), &mut slots).expect("Snapshots");
            seed(&mut snapshots, context, 5);
            let mut prediction_slots = [PredictionSlot::default()];
            let mut prediction =
                PredictionHistory::new(epoch, profile, &mut prediction_slots).expect("Prediction");
            retain(&mut prediction, 1, 999); // Exact request is zero, not nearest one.
            let mut finals = Finals::new();
            let saved = finals.clone();
            let mut source = carrier(context.profile(), 2, false);
            if late {
                *source.last_mut().expect("EOF") = 0xfe;
            }
            let mut attempt = Attempt::new(&source, context);
            let selected = MixedSnapshotHistoryRequest {
                prediction: SnapshotPrediction::History(prediction.source()),
                ..request(context, &snapshots)
            };
            let error = attempt
                .prepare(&mut outgoing, selected, &mut finals)
                .expect_err("Closed failure");
            assert_eq!(error, MixedSnapshotAcknowledgementError::Payload(expected));
            assert_eq!(finals, saved);
            assert_eq!(
                (outgoing.acknowledged_sequence(), outgoing.pending_count()),
                (0, 1)
            );
            assert_eq!(
                outgoing.command(1).expect("Original key").as_bytes(),
                b"PrivateKey"
            );
            assert_eq!(snapshots.newest_sequence(), Some(5));
            assert_eq!(prediction.len(), 1);
            let text = format!("{error:?} {error} {:?}", prediction.source());
            for private in ["PrivateKey", "999", "command_time", "origin"] {
                assert!(!text.contains(private));
            }
        }
    }
}
