// SPDX-License-Identifier: AGPL-3.0-or-later
use super::payload_vectors::carrier;
use super::{contexts, dirty, input, output, sample, values};
use crate::service_support::Fixture;
use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4xr_client_core::{
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    server_payload::{ServerPayloadDecoder, SnapshotDecodeStorage, SnapshotPayloadRequest},
    service_controls::ServiceControlBudget,
    snapshots::services::SnapshotPrediction,
};

#[test]
fn original_ack_owner_stays_unchanged_on_drop_then_commits_after_host_inspection() {
    for context in contexts() {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let mut history = LegacyClientCommands::new(&mut raw).expect("Original owner");
        history.enqueue(b"SnapshotClientKey").expect("Key");
        history.mark_transmitted(1).expect("Sent");
        let fixture = Fixture::new(&carrier(true), 1, b"SnapshotClientKey");
        let payload_codec = ServerPayloadDecoder::new(context.profile(), 0x1234_5678);
        let mut work = vec![0; MAX_MESSAGE_BYTES];
        let mut decoded = vec![0; MAX_MESSAGE_BYTES];
        let (mut entities, mut clients) = dirty();
        let request = SnapshotPayloadRequest {
            input: input(context, 7, None),
            services: ServiceControlBudget::default(),
            prediction: SnapshotPrediction::Transmitted,
        };
        let pending = payload_codec
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
            .expect("Prepared original owner");
        assert_eq!(pending.plan().client_command_acknowledge(), 1);
        assert_eq!(
            values(pending.plan().services().snapshot()),
            (vec![(1, 13)], vec![(63, 511)])
        );
        drop(pending);
        assert_eq!(history.acknowledged_sequence(), 0);
        assert_eq!(history.pending_count(), 1);
        let pending = payload_codec
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
            .expect("Retry");
        let plan = pending.commit();
        assert_eq!(history.acknowledged_sequence(), 1);
        assert_eq!(history.pending_count(), 0);
        history
            .enqueue(b"Replacement")
            .expect("Reused pending slot");
        assert_eq!(
            history
                .command(1)
                .expect("Canonical retained key")
                .as_bytes(),
            b"SnapshotClientKey"
        );
        assert_eq!(plan.message_sequence(), 7);
    }
}
#[test]
fn prepared_ack_outlives_wire_workspaces_and_prediction_sample_before_commit() {
    for context in contexts() {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let mut history = LegacyClientCommands::new(&mut raw).expect("Original owner");
        history.enqueue(b"PrivateSnapshotKey").expect("Key");
        history.mark_transmitted(1).expect("Sent");
        let (mut entities, mut clients) = dirty();
        let pending = {
            let fixture = Fixture::new(&carrier(false), 1, b"PrivateSnapshotKey");
            let payload_codec = ServerPayloadDecoder::new(context.profile(), 0x1234_5678);
            let mut work = vec![0; MAX_MESSAGE_BYTES];
            let mut decoded = vec![0; MAX_MESSAGE_BYTES];
            let sample = sample(context.profile());
            payload_codec
                .prepare_snapshot_ack(
                    &mut history,
                    7,
                    fixture.packet(),
                    SnapshotDecodeStorage {
                        work: &mut work,
                        decoded: &mut decoded,
                    },
                    SnapshotPayloadRequest {
                        input: input(context, 7, None),
                        services: ServiceControlBudget::default(),
                        prediction: SnapshotPrediction::Sample(&sample),
                    },
                    output(&mut entities, &mut clients),
                )
                .expect("Owned cooked plan")
        };
        let debug = format!("{pending:?}");
        assert!(!debug.contains("PrivateSnapshotKey"));
        assert!(!debug.contains("1985229328"));
        let plan = pending.commit();
        assert_eq!(plan.client_command_acknowledge(), 1);
        assert_eq!(history.acknowledged_sequence(), 1);
    }
}
#[test]
fn initial_ack_commits_explicit_native_fallback_without_pending_commands() {
    for context in contexts() {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let mut history = LegacyClientCommands::new(&mut raw).expect("Original owner");
        let fixture = Fixture::new(&carrier(false), 0, b"");
        let payload_codec = ServerPayloadDecoder::new(context.profile(), 0x1234_5678);
        let mut work = vec![0; MAX_MESSAGE_BYTES];
        let mut decoded = vec![0; MAX_MESSAGE_BYTES];
        let (mut entities, mut clients) = dirty();
        let plan = payload_codec
            .prepare_snapshot_ack(
                &mut history,
                7,
                fixture.packet(),
                SnapshotDecodeStorage {
                    work: &mut work,
                    decoded: &mut decoded,
                },
                SnapshotPayloadRequest {
                    input: input(context, 7, None),
                    services: ServiceControlBudget::default(),
                    prediction: SnapshotPrediction::NativePriorFallback,
                },
                output(&mut entities, &mut clients),
            )
            .expect("Explicit fallback")
            .commit();
        assert_eq!(plan.client_command_acknowledge(), 0);
        assert_eq!(history.acknowledged_sequence(), 0);
        assert_eq!(history.pending_count(), 0);
    }
}
