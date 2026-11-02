// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    service_support::{Fixture, seed_identities},
    snapshots::{
        EntityDecodeContext, SnapshotClientSlot,
        mixed_support::{Finals, LIMITS, carrier, selected},
    },
};
use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4xr_client_core::{
    BaselineProtocol,
    client_identities::{ClientIdentityBudget, ClientIdentitySlot, ClientIdentityState},
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    server_commands::{SERVER_COMMAND_SLOT_BYTES, ServerCommandHistory, ServerCommandSlot},
    server_payload::{MixedSnapshotRequest, ServerPayloadDecoder, SnapshotDecodeStorage},
    service_controls::ClientDataUpdateSlot,
    snapshots::{
        history::{SnapshotHistory, SnapshotHistoryError, SnapshotHistorySlot},
        services::SnapshotPrediction,
    },
};

#[test]
fn snapshot_storage_failure_cancels_prepared_command_identity_and_original_ack_owners() {
    let context = EntityDecodeContext::Cod4x21;
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut outgoing = LegacyClientCommands::new(&mut raw).expect("Original outgoing owner");
    outgoing.enqueue(b"PrivateKey").expect("Original key");
    outgoing.mark_transmitted(1).expect("Actual host admission");
    let fixture = Fixture::new(&carrier(context.profile(), 0, true), 1, b"PrivateKey");
    let payload_codec = ServerPayloadDecoder::new(context.profile(), 0x1234_5678);
    let mut work = vec![0; MAX_MESSAGE_BYTES];
    let mut decoded = vec![0; MAX_MESSAGE_BYTES];
    let mut finals = Finals::new();
    let mut text = [0; 128];
    let mut updates = vec![ClientDataUpdateSlot::default(); 4];
    let pending = payload_codec
        .prepare_mixed_snapshot_ack(
            &mut outgoing,
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
        .expect("Complete cooked message");
    let mut command_bytes = vec![0; 2 * SERVER_COMMAND_SLOT_BYTES];
    let mut command_slots = [ServerCommandSlot::default(); 2];
    let mut commands =
        ServerCommandHistory::new(context.profile(), &mut command_bytes, &mut command_slots)
            .expect("Incoming commands");
    let mut cells = vec![ClientIdentitySlot::default(); 2];
    let mut identities =
        ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut cells).expect("Identities");
    seed_identities(&mut identities, 0);
    let mut identity_work = vec![ClientIdentitySlot::default(); 2];
    let mut retained_clients = [SnapshotClientSlot::default()];
    let mut slots = [SnapshotHistorySlot::new(&mut [], &mut retained_clients)];
    let mut snapshots =
        SnapshotHistory::new(7, context.profile(), &mut slots).expect("History owner");
    let prepared_commands = commands
        .prepare(pending.plan().controls())
        .expect("Command resources");
    let prepared_identities = identities
        .prepare_service_plan(
            pending.plan().controls(),
            ClientIdentityBudget {
                max_clients: 2,
                max_records: 2,
            },
            &mut identity_work,
        )
        .expect("Identity resources");
    assert_eq!(
        snapshots
            .prepare(pending.plan().snapshot())
            .expect_err("History storage cannot fit merged entities"),
        SnapshotHistoryError::EntityCapacity
    );
    drop(prepared_commands);
    drop(prepared_identities);
    drop(pending);
    assert_eq!(
        (commands.newest_sequence(), commands.last_message_sequence()),
        (0, 0)
    );
    assert_eq!(identities.accepted_sequence(), Some(0));
    assert!(identities.is_empty());
    assert!(snapshots.is_empty());
    assert_eq!(snapshots.newest_sequence(), None);
    assert_eq!(
        (outgoing.acknowledged_sequence(), outgoing.pending_count()),
        (0, 1)
    );
    assert_eq!(
        outgoing
            .command(1)
            .expect("Still retained original key")
            .as_bytes(),
        b"PrivateKey"
    );
}
