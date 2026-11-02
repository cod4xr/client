// SPDX-License-Identifier: AGPL-3.0-or-later
mod host;
mod receive;
mod wire;

use crate::{
    channel_support::acknowledged,
    mixed_services::support::{seed, send},
    support::server,
};
use cod4xr_client_core::{
    BaselineProtocol,
    client_identities::{ClientIdentitySlot, ClientIdentityState},
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    server_commands::{SERVER_COMMAND_SLOT_BYTES, ServerCommandHistory, ServerCommandSlot},
    server_payload::MixedSnapshotAcknowledgementError,
    service_controls::ServiceControlError,
    snapshot_clients::SnapshotClientSlot,
    snapshot_entities::SnapshotEntitySlot,
    snapshots::{
        history::{SnapshotHistory, SnapshotHistorySlot},
        mixed_services::MixedSnapshotError,
    },
};
use receive::{Finals, receive};
use std::{sync::mpsc, thread};

#[test]
fn mixed_snapshots_cross_scoped_udp_receive_and_commit_all_owners_before_next_control() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let (proceed, wait) = mpsc::channel();
    let responder = thread::spawn(move || wire::serve(&socket, &wait));
    let mut assembly = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut assembly)
        .expect("Original channel");
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut outgoing = LegacyClientCommands::new(&mut raw).expect("Outgoing owner");
    outgoing.enqueue(b"a%").expect("Original key");
    let mut command_bytes = vec![0; 2 * SERVER_COMMAND_SLOT_BYTES];
    let mut command_slots = [ServerCommandSlot::default(); 2];
    let mut commands = ServerCommandHistory::new(
        BaselineProtocol::Cod4x21,
        &mut command_bytes,
        &mut command_slots,
    )
    .expect("Command owner");
    let mut identity_cells = vec![ClientIdentitySlot::default(); 2];
    let mut identities = ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut identity_cells)
        .expect("Identity owner");
    seed(&mut identities);
    send(&mut channel, &mut outgoing, &commands, &identities);
    let mut retained_entities = [SnapshotEntitySlot::default()];
    let mut retained_clients = [SnapshotClientSlot::default()];
    let mut snapshot_slots = [SnapshotHistorySlot::new(
        &mut retained_entities,
        &mut retained_clients,
    )];
    let mut snapshots = SnapshotHistory::new(1, BaselineProtocol::Cod4x21, &mut snapshot_slots)
        .expect("Snapshot history");
    let mut finals = Finals::new();
    let saved = finals.clone();
    assert_eq!(
        receive(
            &mut channel,
            &mut outgoing,
            commands.newest_sequence(),
            &snapshots,
            &mut finals
        )
        .expect_err("Late malformed service"),
        MixedSnapshotAcknowledgementError::Payload(MixedSnapshotError::Controls(
            ServiceControlError::UnsupportedService(0xfe)
        ))
    );
    assert_eq!(finals, saved);
    assert_eq!(
        (outgoing.acknowledged_sequence(), outgoing.pending_count()),
        (0, 1)
    );
    assert_eq!(commands.newest_sequence(), 0);
    assert_eq!(identities.accepted_sequence(), Some(9));
    assert!(identities.is_empty());
    assert_eq!(channel.split().receiver.incoming_sequence(), 7);
    proceed.send(()).expect("Allow valid next message");
    for (sequence, prefix, acknowledgement, text) in [
        (8, 3, 1, b"next".as_slice()),
        (11, 4, 2, b"last".as_slice()),
    ] {
        let pending = receive(
            &mut channel,
            &mut outgoing,
            commands.newest_sequence(),
            &snapshots,
            &mut finals,
        )
        .expect("Header-selected mixed snapshot");
        assert!(!format!("{pending:?}").contains("Private"));
        assert!(!format!("{pending:?}").contains("Before"));
        let plan = host::commit_after_resource_retry(
            pending,
            &mut commands,
            &mut identities,
            &mut snapshots,
        );
        host::assert_owners(&outgoing, &commands, &identities, sequence, acknowledgement);
        host::assert_snapshot(&plan, prefix);
        assert_eq!(
            snapshots
                .source(sequence)
                .expect("Committed whole snapshot")
                .message(),
            plan.snapshot().message()
        );
        outgoing
            .enqueue(text)
            .expect("Reuse acknowledged pending storage");
        send(&mut channel, &mut outgoing, &commands, &identities);
    }
    assert!(snapshots.source(8).is_none());
    assert_eq!(snapshots.newest_sequence(), Some(11));
    responder
        .join()
        .expect("Peer verified actual outgoing context");
    assert_eq!(channel.receive_usage().messages, 3);
}
