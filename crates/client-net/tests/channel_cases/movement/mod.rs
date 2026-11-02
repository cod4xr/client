// SPDX-License-Identifier: AGPL-3.0-or-later
mod receive;
mod wire;

use crate::{channel_support::acknowledged, support::server};
use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    control::{ClientControlHeader, ControlMessagePlan, MovementBatch},
    player_state::{PredictionSample, PredictionWords},
    server_commands::{SERVER_COMMAND_SLOT_BYTES, ServerCommandHistory, ServerCommandSlot},
    user_commands::{MovementAxis, UserCommandInput},
};
use std::{
    thread,
    time::{Duration, Instant},
};

#[test]
fn owned_movement_uses_original_socket_context_and_admitted_high_byte_key_before_peer_ack() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || wire::serve(&socket));
    let mut assembly = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut assembly)
        .expect("Original channel");
    let mut outgoing_bytes = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut outgoing = LegacyClientCommands::new(&mut outgoing_bytes).expect("Outgoing owner");
    outgoing
        .enqueue(b"reliable")
        .expect("Pending reliable command");
    let mut incoming_bytes = vec![0; 2 * SERVER_COMMAND_SLOT_BYTES];
    let mut incoming_slots = [ServerCommandSlot::default(); 2];
    let mut incoming = ServerCommandHistory::new(
        BaselineProtocol::Cod4x21,
        &mut incoming_bytes,
        &mut incoming_slots,
    )
    .expect("Incoming owner");
    receive::admit(&mut channel, &mut outgoing, &mut incoming);
    assert_eq!(
        (incoming.last_message_sequence(), incoming.newest_sequence()),
        (7, 9)
    );
    assert_eq!(
        incoming
            .latest_key()
            .expect("Admitted high-byte key")
            .as_bytes(),
        b"\x92"
    );
    let input = commands();
    let header = ClientControlHeader::Cod4x21 {
        server_id: 0x1122_3344,
        message_acknowledge: incoming.last_message_sequence(),
        server_command: incoming.latest_key().expect("Retained original key"),
        config_acknowledge: 0,
    };
    let report = prediction();
    let plan = ControlMessagePlan::with_movement(
        header,
        channel.context().server_challenge(),
        &outgoing,
        MovementBatch {
            commands: &input,
            checksum_feed: wire::FEED,
            prediction: report,
            snapshot: None,
        },
    )
    .expect("Owned movement frame");
    assert_eq!((plan.command_count(), plan.user_command_count()), (1, 3));
    assert_eq!(
        (
            outgoing.transmitted_sequence(),
            outgoing.acknowledged_sequence()
        ),
        (0, 0)
    );
    {
        let mut sender = channel.split().sender;
        let mut message = sender
            .begin_control(&plan)
            .expect("Same original context guard");
        let deadline = Instant::now() + Duration::from_secs(3);
        while message
            .send_next(deadline)
            .expect("Actual packet I/O")
            .is_some()
        {}
    }
    outgoing
        .mark_transmitted(
            plan.last_command_sequence()
                .expect("Included reliable prefix"),
        )
        .expect("Only after complete I/O");
    assert_eq!(
        (
            outgoing.transmitted_sequence(),
            outgoing.acknowledged_sequence()
        ),
        (1, 0)
    );
    receive::admit(&mut channel, &mut outgoing, &mut incoming);
    assert_eq!(
        (outgoing.acknowledged_sequence(), outgoing.pending_count()),
        (1, 0)
    );
    assert_eq!(
        (incoming.last_message_sequence(), incoming.newest_sequence()),
        (8, 9)
    );
    responder
        .join()
        .expect("Peer decoded actual commands and acknowledged");
    assert_eq!(channel.receive_usage().messages, 2);
}

fn commands() -> Vec<UserCommandInput> {
    (0_u8..3)
        .map(|index| UserCommandInput {
            server_time: 100 + i32::from(index) * 16,
            buttons: 1,
            angles: [0x1234, 0x8000, 0xffff],
            weapon: 31 + index,
            offhand: 0,
            movement: [MovementAxis::Negative, MovementAxis::Positive],
            selected_location: None,
            melee: None,
        })
        .collect()
}

fn prediction() -> PredictionSample {
    PredictionSample::new(
        BaselineProtocol::Cod4x21,
        132,
        PredictionWords::new(wire::ORIGIN, [0; 3], [0; 3], 0, 0),
    )
}
