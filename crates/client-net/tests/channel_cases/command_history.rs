// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    net::UdpSocket,
    thread,
    time::{Duration, Instant},
};

use cod4x_protocol::{MessageReader, MessageStringLimit, compression, obfuscation};
use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    control::{ClientControlHeader, ControlMessagePlan},
    server_commands::{SERVER_COMMAND_SLOT_BYTES, ServerCommandHistory, ServerCommandSlot},
    service_controls::{LegacyServicePlan, ServiceControlBudget},
};
use cod4xr_client_net::{ChannelEvent, ClientChannel};

use super::services::{response, send_control};
use crate::{
    channel_support::{accept, acknowledged},
    support::{receive, server},
};

fn serve(socket: &UdpSocket) {
    let client = accept(socket);
    let (first, source) = receive(socket);
    assert_eq!(source, client);
    assert_eq!(&first[..4], &1_u32.to_le_bytes());
    response(socket, client, 7, b"\x04\x03\0\0\0%PrivateStored\0\x07");
    let (next, source) = receive(socket);
    assert_eq!(source, client);
    assert_eq!(&next[..4], &2_u32.to_le_bytes());
    assert_eq!(&next[4..6], &40_000_u16.to_le_bytes());
    assert_eq!(
        &next[6..22],
        &[0, 0, 0, 0, 7, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0]
    );
    let mut body = next[22..].to_vec();
    obfuscation::apply_client_payload(
        &mut body,
        u32::from_le_bytes((-2_147_481_234_i32).to_le_bytes()),
        0,
        7,
        b".PrivateStored",
    )
    .expect("New normalized retained server key");
    let mut decoded = [0; 1024];
    let length =
        compression::decompress(&body, &mut decoded).expect("Control with latest server key");
    let mut reader = MessageReader::try_new(&decoded[..length]).expect("Client control");
    assert_eq!(reader.read_bits(3).expect("Command opcode"), 2);
    assert_eq!(reader.read_i32().expect("Next command sequence"), 2);
    let mut text = [0xa5; 32];
    let length = reader
        .read_legacy_string(&mut text, MessageStringLimit::Normal)
        .expect("Complete next command");
    assert_eq!(&text[..length], b"next");
    assert_eq!(reader.read_bits(3).expect("Complete EOF"), 3);
}

pub(super) fn send_next(
    channel: &mut ClientChannel<'_>,
    client: &mut LegacyClientCommands<'_>,
    server: &ServerCommandHistory<'_>,
) {
    let header = ClientControlHeader::Cod4x21 {
        server_id: 0,
        message_acknowledge: server.last_message_sequence(),
        server_command: server.latest_key().expect("Latest retained key"),
        config_acknowledge: 0,
    };
    let frame = ControlMessagePlan::new(header, channel.context().server_challenge(), client)
        .expect("Updated control frame");
    {
        let mut sender = channel.split().sender;
        let mut message = sender.begin_control(&frame).expect("Same original context");
        let deadline = Instant::now() + Duration::from_secs(2);
        while message.send_next(deadline).expect("Outgoing I/O").is_some() {}
    }
    client
        .mark_transmitted(
            frame
                .last_command_sequence()
                .expect("Included next command"),
        )
        .expect("Complete frame admitted");
}

#[test]
fn retained_server_key_and_validated_message_ack_drive_the_next_bidirectional_control_frame() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || serve(&socket));
    let mut reassembly = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut reassembly)
        .expect("Channel");
    let decoder = channel.server_decoder().expect("Original decoder");
    let mut client_data = [0; 3 * LEGACY_COMMAND_SLOT_BYTES];
    let mut client = LegacyClientCommands::new(&mut client_data).expect("Client commands");
    client.enqueue(b"a%").expect("First command key");
    send_control(&mut channel, &mut client);
    let (sequence, body) = {
        let mut receiver = channel.split().receiver;
        // Test ownership crosses the scoped callback through an explicit bounded copy.
        receiver
            .receive_until(
                Instant::now() + Duration::from_secs(2),
                |event| match event {
                    ChannelEvent::Message(message) => {
                        Some((message.sequence(), message.body().to_vec()))
                    }
                    _ => None,
                },
            )
            .expect("Receive")
            .expect("One datagram")
            .expect("Ordinary body")
    };
    let mut work = [0; 1024];
    let mut service_bytes = [0; 1024];
    let mut output = [0; 128];
    let plan = {
        let payload = decoder
            .decode(sequence, &body, &client, &mut work, &mut service_bytes)
            .expect("Payload");
        LegacyServicePlan::parse(
            &payload,
            ServiceControlBudget::default(),
            &mut [0; 128],
            &mut output,
        )
        .expect("Complete services")
    };
    let mut data = vec![0xa5; 3 * SERVER_COMMAND_SLOT_BYTES];
    let mut slots = [ServerCommandSlot::default(); 3];
    let mut retained = ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut data, &mut slots)
        .expect("Budgeted retained history");
    let prepared = retained
        .prepare(&plan)
        .expect("Incoming resources preflighted");
    client
        .commit_acknowledgement(plan.client_command_acknowledge())
        .expect("Validated client acknowledgement");
    let summary = prepared.commit();
    assert_eq!((summary.new_commands, summary.skipped_sequences), (1, 2));
    assert_eq!(
        retained
            .command(3)
            .expect("Full normalized record")
            .as_bytes(),
        b".PrivateStored"
    );
    assert_eq!(
        (retained.last_message_sequence(), client.pending_count()),
        (7, 0)
    );
    client.enqueue(b"next").expect("Next queued command");
    send_next(&mut channel, &mut client, &retained);
    responder.join().expect("Responder");
    assert_eq!(
        (
            client.transmitted_sequence(),
            client.acknowledged_sequence(),
            client.pending_count()
        ),
        (2, 1, 1)
    );
    assert_eq!(channel.send_usage().messages_reserved, 2);
}
