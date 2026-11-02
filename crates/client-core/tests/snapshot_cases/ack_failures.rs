// SPDX-License-Identifier: AGPL-3.0-or-later
use super::payload_vectors::carrier;
use super::{
    PlayerStateError, PredictionSample, SnapshotError, contexts, dirty, input, output, sample,
};
use crate::service_support::Fixture;
use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4xr_client_core::{
    commands::{ClientCommandError, LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    server_payload::{
        ServerPayloadDecoder, ServerPayloadError, SnapshotAcknowledgementError,
        SnapshotDecodeStorage, SnapshotPayloadRequest,
    },
    service_controls::ServiceControlBudget,
    snapshots::services::{SnapshotPayloadError, SnapshotPrediction, SnapshotServiceError},
};

#[test]
fn malformed_shared_headers_reject_before_history_and_cooked_output_mutation() {
    for context in contexts() {
        for (sequence, packet, expected) in [
            (0, vec![], ServerPayloadError::Sequence),
            (7, vec![0; 3], ServerPayloadError::Header),
            (7, vec![0; 4], ServerPayloadError::Empty),
            (
                7,
                vec![0; MAX_MESSAGE_BYTES + 1],
                ServerPayloadError::InputCapacity,
            ),
        ] {
            let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
            let mut history = LegacyClientCommands::new(&mut raw).expect("Owner");
            let payload_codec = ServerPayloadDecoder::new(context.profile(), 0x1234_5678);
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            let request = SnapshotPayloadRequest {
                input: input(context, 7, None),
                services: ServiceControlBudget::default(),
                prediction: SnapshotPrediction::Transmitted,
            };
            let error = payload_codec
                .prepare_snapshot_ack(
                    &mut history,
                    sequence,
                    &packet,
                    SnapshotDecodeStorage {
                        work: &mut [],
                        decoded: &mut [],
                    },
                    request,
                    output(&mut entities, &mut clients),
                )
                .expect_err("Header failure");
            assert_eq!(error, SnapshotAcknowledgementError::Decode(expected));
            assert_eq!(history.acknowledged_sequence(), 0);
            assert_eq!((entities, clients), saved);
        }
    }
}
#[test]
fn unsent_ack_and_insufficient_private_decode_workspace_leave_original_key_retained() {
    for context in contexts() {
        for unsent in [false, true] {
            let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
            let mut history = LegacyClientCommands::new(&mut raw).expect("Owner");
            history.enqueue(b"SnapshotClientKey").expect("Key");
            history.mark_transmitted(1).expect("Sent");
            let fixture = Fixture::new(
                &carrier(true),
                if unsent { 2 } else { 1 },
                b"SnapshotClientKey",
            );
            let payload_codec = ServerPayloadDecoder::new(context.profile(), 0x1234_5678);
            let mut decoded = vec![0; MAX_MESSAGE_BYTES];
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            let request = SnapshotPayloadRequest {
                input: input(context, 7, None),
                services: ServiceControlBudget::default(),
                prediction: SnapshotPrediction::Transmitted,
            };
            let error = payload_codec
                .prepare_snapshot_ack(
                    &mut history,
                    7,
                    fixture.packet(),
                    SnapshotDecodeStorage {
                        work: &mut [],
                        decoded: &mut decoded,
                    },
                    request,
                    output(&mut entities, &mut clients),
                )
                .expect_err("Pre-publication failure");
            assert_eq!(
                error,
                if unsent {
                    SnapshotAcknowledgementError::Acknowledgement(
                        ClientCommandError::UnsentAcknowledgement,
                    )
                } else {
                    SnapshotAcknowledgementError::Decode(ServerPayloadError::WorkCapacity)
                }
            );
            assert_eq!(history.acknowledged_sequence(), 0);
            assert_eq!(history.pending_count(), 1);
            assert_eq!(
                history.command(1).expect("Original key").as_bytes(),
                b"SnapshotClientKey"
            );
            assert_eq!((entities, clients), saved);
        }
    }
}
#[test]
fn late_service_and_prediction_failures_cancel_original_ack_lease_and_both_outputs() {
    for context in contexts() {
        for prediction_failure in [false, true] {
            let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
            let mut history = LegacyClientCommands::new(&mut raw).expect("Owner");
            history.enqueue(b"SnapshotClientKey").expect("Key");
            history.mark_transmitted(1).expect("Sent");
            let mut bytes = carrier(!prediction_failure);
            if !prediction_failure {
                let last = bytes.len() - 1;
                bytes[last] = 11;
            }
            let fixture = Fixture::new(&bytes, 1, b"SnapshotClientKey");
            let payload_codec = ServerPayloadDecoder::new(context.profile(), 0x1234_5678);
            let mut work = vec![0; MAX_MESSAGE_BYTES];
            let mut decoded = vec![0; MAX_MESSAGE_BYTES];
            let wrong =
                PredictionSample::new(context.profile(), 1, sample(context.profile()).words());
            let request = SnapshotPayloadRequest {
                input: input(context, 7, None),
                services: ServiceControlBudget::default(),
                prediction: if prediction_failure {
                    SnapshotPrediction::Sample(&wrong)
                } else {
                    SnapshotPrediction::Transmitted
                },
            };
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            let error = payload_codec
                .prepare_snapshot_ack(
                    &mut history,
                    7,
                    fixture.packet(),
                    SnapshotDecodeStorage {
                        work: &mut work,
                        decoded: &mut decoded,
                    },
                    request,
                    output(&mut entities, &mut clients),
                )
                .expect_err("Late validation failure");
            let expected = if prediction_failure {
                SnapshotServiceError::Snapshot(SnapshotError::Player(
                    PlayerStateError::PredictionTime,
                ))
            } else {
                SnapshotServiceError::UnsupportedService(11)
            };
            assert_eq!(
                error,
                SnapshotAcknowledgementError::Payload(SnapshotPayloadError::Services(expected))
            );
            assert_eq!(history.acknowledged_sequence(), 0);
            assert_eq!(history.pending_count(), 1);
            assert_eq!((entities, clients), saved);
        }
    }
}
