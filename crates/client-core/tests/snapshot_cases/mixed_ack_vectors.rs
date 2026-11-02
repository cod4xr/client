// SPDX-License-Identifier: AGPL-3.0-or-later
use super::mixed_support::{Finals, LIMITS, carrier, selected};
use super::{EntityDecodeContext, SnapshotMessage, contexts};
use crate::service_support::{Fixture, seed_identities};
use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4xr_client_core::{
    BaselineProtocol,
    client_identities::{ClientIdentityBudget, ClientIdentitySlot, ClientIdentityState},
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    server_commands::{SERVER_COMMAND_SLOT_BYTES, ServerCommandHistory, ServerCommandSlot},
    server_payload::{MixedSnapshotRequest, ServerPayloadDecoder, SnapshotDecodeStorage},
    service_controls::ClientDataUpdateSlot,
    snapshots::{
        history::{SnapshotHistory, SnapshotHistorySlot},
        services::SnapshotPrediction,
    },
};

#[test]
fn original_mixed_ack_lease_outlives_all_scratch_and_cancels_before_retry_commit() {
    for context in contexts() {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let mut history = LegacyClientCommands::new(&mut raw).expect("Original owner");
        history.enqueue(b"PrivateMixedKey").expect("Key");
        history.mark_transmitted(1).expect("Sent");
        let mut finals = Finals::new();
        let pending = {
            let fixture = Fixture::new(&carrier(context.profile(), 0, true), 1, b"PrivateMixedKey");
            let payload_codec = ServerPayloadDecoder::new(context.profile(), 0x1234_5678);
            let mut work = vec![0; MAX_MESSAGE_BYTES];
            let mut decoded = vec![0; MAX_MESSAGE_BYTES];
            let mut text = [0; 128];
            let mut updates = vec![ClientDataUpdateSlot::default(); 4];
            payload_codec
                .prepare_mixed_snapshot_ack(
                    &mut history,
                    7,
                    fixture.packet(),
                    SnapshotDecodeStorage {
                        work: &mut work,
                        decoded: &mut decoded,
                    },
                    MixedSnapshotRequest {
                        input: selected(context, 7),
                        services: LIMITS,
                        prediction: SnapshotPrediction::Transmitted,
                    },
                    finals.storage(&mut text, &mut updates),
                )
                .expect("Prepared mixed owner")
        };
        assert_eq!(pending.plan().controls().client_command_acknowledge(), 1);
        assert_eq!(
            pending
                .plan()
                .snapshot()
                .message()
                .server_command_sequence(),
            4
        );
        let debug = format!("{pending:?}");
        assert!(!debug.contains("PrivateMixedKey"));
        assert!(!debug.contains("Before"));
        drop(pending);
        assert_eq!(history.acknowledged_sequence(), 0);
        assert_eq!(history.pending_count(), 1);
        let fixture = Fixture::new(&carrier(context.profile(), 0, true), 1, b"PrivateMixedKey");
        let payload_codec = ServerPayloadDecoder::new(context.profile(), 0x1234_5678);
        let mut work = vec![0; MAX_MESSAGE_BYTES];
        let mut decoded = vec![0; MAX_MESSAGE_BYTES];
        let mut text = [0; 128];
        let mut updates = vec![ClientDataUpdateSlot::default(); 4];
        let plan = payload_codec
            .prepare_mixed_snapshot_ack(
                &mut history,
                7,
                fixture.packet(),
                SnapshotDecodeStorage {
                    work: &mut work,
                    decoded: &mut decoded,
                },
                MixedSnapshotRequest {
                    input: selected(context, 7),
                    services: LIMITS,
                    prediction: SnapshotPrediction::Transmitted,
                },
                finals.storage(&mut text, &mut updates),
            )
            .expect("Retry")
            .commit();
        assert_eq!(history.acknowledged_sequence(), 1);
        history
            .enqueue(b"Replacement")
            .expect("Reused pending slot");
        assert_eq!(
            history
                .command(1)
                .expect("Retained canonical key")
                .as_bytes(),
            b"PrivateMixedKey"
        );
        assert_eq!(plan.controls().command_count(), 3);
    }
}
#[test]
fn all_incoming_owners_preflight_then_commit_before_original_outgoing_ack_release() {
    let context = EntityDecodeContext::Cod4x21;
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut raw).expect("Original owner");
    history.enqueue(b"PrivateMixedKey").expect("Key");
    history.mark_transmitted(1).expect("Sent");
    let fixture = Fixture::new(&carrier(context.profile(), 0, true), 1, b"PrivateMixedKey");
    let payload_codec = ServerPayloadDecoder::new(context.profile(), 0x1234_5678);
    let mut work = vec![0; MAX_MESSAGE_BYTES];
    let mut decoded = vec![0; MAX_MESSAGE_BYTES];
    let mut finals = Finals::new();
    let mut text = [0; 128];
    let mut updates = vec![ClientDataUpdateSlot::default(); 4];
    let mut server_bytes = vec![0; 2 * SERVER_COMMAND_SLOT_BYTES];
    let mut server_slots = [ServerCommandSlot::default(); 2];
    let mut server =
        ServerCommandHistory::new(context.profile(), &mut server_bytes, &mut server_slots)
            .expect("Incoming command owner");
    let mut current = selected(context, 7);
    current.message =
        SnapshotMessage::new(7, 7, server.newest_sequence()).expect("Actual host command prefix");
    let pending = payload_codec
        .prepare_mixed_snapshot_ack(
            &mut history,
            7,
            fixture.packet(),
            SnapshotDecodeStorage {
                work: &mut work,
                decoded: &mut decoded,
            },
            MixedSnapshotRequest {
                input: current,
                services: LIMITS,
                prediction: SnapshotPrediction::Transmitted,
            },
            finals.storage(&mut text, &mut updates),
        )
        .expect("Complete mixed preparation");
    let mut cells = vec![ClientIdentitySlot::default(); 2];
    let mut identities =
        ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut cells).expect("Identity owner");
    seed_identities(&mut identities, 0);
    let mut identity_work = vec![ClientIdentitySlot::default(); 2];
    let mut retained_entities = vec![super::SnapshotEntitySlot::default(); 2];
    let mut retained_clients = vec![super::SnapshotClientSlot::default(); 2];
    let mut snapshot_slots = [SnapshotHistorySlot::new(
        &mut retained_entities,
        &mut retained_clients,
    )];
    let mut snapshots =
        SnapshotHistory::new(7, context.profile(), &mut snapshot_slots).expect("Snapshot owner");
    let commands = server
        .prepare(pending.plan().controls())
        .expect("Command preflight");
    let patches = identities
        .prepare_service_plan(
            pending.plan().controls(),
            ClientIdentityBudget {
                max_clients: 2,
                max_records: 2,
            },
            &mut identity_work,
        )
        .expect("Identity preflight");
    let retained = snapshots
        .prepare(pending.plan().snapshot())
        .expect("Snapshot resource preflight");
    let command_update = commands.commit();
    let _ = patches.commit();
    let _ = retained.commit();
    let plan = pending.commit();
    assert_eq!(command_update.newest_command, 9);
    assert_eq!(history.acknowledged_sequence(), 1);
    assert_eq!(identities.accepted_sequence(), Some(2));
    assert_eq!(identities.client(2).expect("Identity").name(), b"After");
    assert_eq!(plan.snapshot().message().server_command_sequence(), 4);
    assert_eq!(
        snapshots
            .source(7)
            .expect("Retained original snapshot")
            .message(),
        plan.snapshot().message()
    );
}
