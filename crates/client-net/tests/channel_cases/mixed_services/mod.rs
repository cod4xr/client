// SPDX-License-Identifier: AGPL-3.0-or-later
mod support;
use crate::{channel_support::acknowledged, support::server};
use cod4xr_client_core::{
    BaselineProtocol,
    client_identities::{ClientIdentitySlot, ClientIdentityState},
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    server_commands::{SERVER_COMMAND_SLOT_BYTES, ServerCommandHistory, ServerCommandSlot},
    service_controls::{ClientDataUpdateSlot, ServiceControlError},
};
use std::{sync::mpsc, thread};

#[test]
fn whole_mixed_udp_messages_drive_command_and_config_ack_only_after_all_application_preflight() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let (proceed, wait) = mpsc::channel();
    let responder = thread::spawn(move || support::serve(&socket, &wait));
    let mut assembly = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut assembly)
        .expect("Original channel");
    let mut outgoing_bytes = [0; 3 * LEGACY_COMMAND_SLOT_BYTES];
    let mut outgoing = LegacyClientCommands::new(&mut outgoing_bytes).expect("Outgoing");
    outgoing.enqueue(b"a%").expect("Initial key");
    let mut command_bytes = vec![0; 2 * SERVER_COMMAND_SLOT_BYTES];
    let mut command_slots = [ServerCommandSlot::default(); 2];
    let mut commands = ServerCommandHistory::new(
        BaselineProtocol::Cod4x21,
        &mut command_bytes,
        &mut command_slots,
    )
    .expect("Retained commands");
    let mut identity_cells = vec![ClientIdentitySlot::default(); 2];
    let mut identities = ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut identity_cells)
        .expect("Identity owner");
    support::seed(&mut identities);
    support::send(&mut channel, &mut outgoing, &commands, &identities);
    let mut text = [0xa5; 128];
    let mut updates = vec![ClientDataUpdateSlot::default(); 4];
    let before = updates.clone();
    let mut private = vec![ClientDataUpdateSlot::default(); 4];
    let mut identity_work = vec![ClientIdentitySlot::default(); 2];
    for valid in [false, true] {
        let result = support::receive_plan(
            &mut channel,
            &outgoing,
            &mut private,
            &mut text,
            &mut updates,
        );
        if valid {
            let plan = result.expect("Whole message");
            assert!(!format!("{plan:?}").contains("Private"));
            assert_eq!(outgoing.acknowledged_sequence(), 0);
            let (retained, applied) = support::apply(
                &plan,
                &mut outgoing,
                &mut commands,
                &mut identities,
                &mut identity_work,
            )
            .expect("All resources before ACK");
            assert_eq!(
                (
                    retained.new_commands,
                    applied.accepted_records,
                    applied.ignored_records
                ),
                (1, 2, 1)
            );
            assert_eq!(outgoing.acknowledged_sequence(), 1);
            assert_eq!(identities.accepted_sequence(), Some(11));
            assert_eq!(commands.last_message_sequence(), 8);
        } else {
            assert!(matches!(
                result,
                Err(ServiceControlError::UnsupportedService(0xfe))
            ));
            assert_eq!(text, [0xa5; 128]);
            assert_eq!(updates, before);
            assert_eq!(
                (outgoing.acknowledged_sequence(), outgoing.pending_count()),
                (0, 1)
            );
            assert_eq!(commands.newest_sequence(), 0);
            assert_eq!(identities.accepted_sequence(), Some(9));
            assert!(identities.is_empty());
            proceed.send(()).expect("Next wire message");
        }
    }
    outgoing.enqueue(b"next").expect("Next command");
    support::send(&mut channel, &mut outgoing, &commands, &identities);
    responder.join().expect("Peer");
    updates.fill(ClientDataUpdateSlot::default());
    private.fill(ClientDataUpdateSlot::default());
    identity_work.fill(ClientIdentitySlot::default());
    assert_eq!(
        identities
            .client(3)
            .expect("Retained normalized identity")
            .name(),
        b"Player."
    );
    assert_eq!(
        identities.client(63).expect("Raw retained identity").name(),
        b"Other\xff"
    );
    assert_eq!(
        commands.command(3).expect("Retained key").as_bytes(),
        b"Private.Stored"
    );
    assert_eq!(channel.receive_usage().messages, 2);
}
