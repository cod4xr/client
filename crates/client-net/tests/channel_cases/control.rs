// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    net::UdpSocket,
    thread,
    time::{Duration, Instant},
};

use cod4x_protocol::{MessageReader, MessageStringLimit, compression, obfuscation};
use cod4xr_client_core::{
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    control::{ClientControlHeader, ControlMessagePlan, ServerCommandKey},
};
use cod4xr_client_net::NetworkError;

use crate::{
    channel_support::{accept, acknowledged},
    support::{receive, server},
};

fn serve_control(socket: &UdpSocket) {
    let client = accept(socket);
    let mut body = Vec::new();
    loop {
        let (bytes, source) = receive(socket);
        assert_eq!(source, client);
        assert_eq!(&bytes[..4], &0x8000_0001_u32.to_le_bytes());
        assert_eq!(&bytes[4..6], &40_000_u16.to_le_bytes());
        assert_eq!(
            &bytes[6..10],
            &u32::try_from(body.len())
                .expect("Fragment offset")
                .to_le_bytes()
        );
        let length = usize::from(u16::from_le_bytes([bytes[10], bytes[11]]));
        assert_eq!(bytes.len(), 12 + length);
        body.extend_from_slice(&bytes[12..]);
        if length < 1300 {
            break;
        }
    }
    assert_eq!(
        &body[..16],
        &[0x44, 0x33, 0x22, 0x11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    let cookie = u32::from_le_bytes((-2_147_481_234_i32).to_le_bytes());
    obfuscation::apply_client_payload(&mut body[16..], cookie, 0x1122_3344, 0, b"")
        .expect("Profile-specific transform");
    let mut decoded = [0; 4096];
    let length =
        compression::decompress(&body[16..], &mut decoded).expect("Complete compressed frame");
    let mut reader = MessageReader::try_new(&decoded[..length]).expect("Control message");
    assert_eq!(reader.read_bits(3).expect("Command opcode"), 2);
    assert_eq!(reader.read_i32().expect("Command sequence"), 1);
    let mut command = [0xa5; 1023];
    assert_eq!(
        reader
            .read_legacy_string(&mut command, MessageStringLimit::Normal)
            .expect("Complete private command"),
        1000
    );
    assert_eq!(&command[..1000], &[0xd5; 1000]);
    assert_eq!(reader.read_bits(3).expect("Complete EOF"), 3);
}

#[test]
fn owned_control_frames_check_connection_context_before_reservation_and_send_complete_prefixes() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || serve_control(&socket));
    let mut reassembly = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut reassembly)
        .expect("Channel");
    let cookie = channel.context().server_challenge();
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("History");
    history.enqueue(&[0xd5; 1000]).expect("Private command");
    let modern = ClientControlHeader::Cod4x21 {
        server_id: 0x1122_3344,
        message_acknowledge: 0,
        server_command: ServerCommandKey::initial(),
        config_acknowledge: 0,
    };
    let legacy = ClientControlHeader::Legacy6 {
        server_id: 0,
        message_acknowledge: 0,
        server_command: ServerCommandKey::initial(),
    };
    let wrong_profile =
        ControlMessagePlan::new(legacy, cookie, &history).expect("Different profile");
    let wrong_cookie = ControlMessagePlan::new(modern, cookie.wrapping_add(1), &history)
        .expect("Different cookie");
    let frame = ControlMessagePlan::new(modern, cookie, &history).expect("Original context");
    {
        let mut sender = channel.split().sender;
        for rejected in [&wrong_profile, &wrong_cookie] {
            assert!(matches!(
                sender.begin_control(rejected),
                Err(NetworkError::ControlContext)
            ));
            assert_eq!(sender.next_sequence(), 1);
            assert_eq!(sender.usage().messages_reserved, 0);
        }
        let mut message = sender.begin_control(&frame).expect("Checked context");
        let deadline = Instant::now() + Duration::from_secs(2);
        while message
            .send_next(deadline)
            .expect("Next complete fragment")
            .is_some()
        {
            assert_eq!(history.transmitted_sequence(), 0);
        }
    }
    history
        .mark_transmitted(
            frame
                .last_command_sequence()
                .expect("Included command prefix"),
        )
        .expect("Full message admitted to I/O");
    responder.join().expect("Responder");
    assert_eq!(channel.send_usage().datagrams_sent, 2);
    assert_eq!(history.transmitted_sequence(), 1);
    assert_eq!(history.acknowledged_sequence(), 0);
}
