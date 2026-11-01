// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    net::{SocketAddr, UdpSocket},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use cod4x_protocol::{compression, obfuscation};
use cod4xr_client_core::{
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    control::{ClientControlHeader, ControlMessagePlan, ServerCommandKey},
    service_controls::{LegacyServicePlan, ServiceControlBudget, ServiceControlError},
};
use cod4xr_client_net::{ChannelEvent, ClientChannel};

use crate::{
    channel_support::{accept, acknowledged, ordinary},
    support::{receive, server},
};

fn response(socket: &UdpSocket, client: SocketAddr, sequence: u32, source: &[u8]) {
    let mut body = [0; 2048];
    body[..4].copy_from_slice(&1_u32.to_le_bytes());
    let length = compression::compress(source, &mut body[4..]).expect("Fixture compression");
    obfuscation::apply_server_payload(
        &mut body[4..4 + length],
        u32::from_le_bytes((-2_147_481_234_i32).to_le_bytes()),
        sequence,
        b"a.",
    )
    .expect("Original command key");
    socket
        .send_to(&ordinary(sequence, &body[..4 + length]), client)
        .expect("Server service stream");
}

fn serve(socket: &UdpSocket, proceed: &Receiver<()>) {
    let client = accept(socket);
    let (request, source) = receive(socket);
    assert_eq!(source, client);
    assert_eq!(&request[..4], &1_u32.to_le_bytes());
    assert_eq!(&request[4..6], &40_000_u16.to_le_bytes());
    assert_eq!(&request[6..22], &[0; 16]);
    assert!(request.len() > 22);
    response(socket, client, 7, b"\x04\x03\0\0\0PrivateBad%\0\xfe");
    proceed
        .recv_timeout(Duration::from_secs(4))
        .expect("Malformed service rejected");
    response(socket, client, 8, b"\x04\x03\0\0\0PrivateValid%\0\x07");
}

fn send_control(channel: &mut ClientChannel<'_>, history: &mut LegacyClientCommands<'_>) {
    let header = ClientControlHeader::Cod4x21 {
        server_id: 0,
        message_acknowledge: 0,
        server_command: ServerCommandKey::initial(),
        config_acknowledge: 0,
    };
    let frame = ControlMessagePlan::new(header, channel.context().server_challenge(), history)
        .expect("Client control frame");
    {
        let mut sender = channel.split().sender;
        let mut message = sender.begin_control(&frame).expect("Original connection");
        let deadline = Instant::now() + Duration::from_secs(2);
        while message
            .send_next(deadline)
            .expect("Complete outgoing frame")
            .is_some()
        {}
    }
    history
        .mark_transmitted(frame.last_command_sequence().expect("Included command"))
        .expect("I/O admitted");
}

#[test]
fn malformed_services_do_not_release_pending_commands_and_complete_eof_allows_host_commit() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let (proceed, wait) = mpsc::channel();
    let responder = thread::spawn(move || serve(&socket, &wait));
    let mut reassembly = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut reassembly)
        .expect("Channel");
    let decoder = channel.server_decoder().expect("Connection decoder");
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("History");
    history.enqueue(b"a%").expect("Private canonical command");
    send_control(&mut channel, &mut history);
    let mut work = [0xa5; 1024];
    let mut service_bytes = [0xa5; 1024];
    let mut scratch = [0xa5; 128];
    let mut output = [0xa5; 128];
    let mut validated_message = 0;
    {
        let mut receiver = channel.split().receiver;
        let deadline = Instant::now() + Duration::from_secs(2);
        for valid in [false, true] {
            let result = receiver
                .receive_until(deadline, |event| {
                    let ChannelEvent::Message(message) = event else {
                        return None;
                    };
                    let payload = decoder
                        .decode(
                            message.sequence(),
                            message.body(),
                            &history,
                            &mut work,
                            &mut service_bytes,
                        )
                        .expect("Payload before service validation");
                    Some(
                        LegacyServicePlan::parse(
                            &payload,
                            ServiceControlBudget::default(),
                            &mut scratch,
                            &mut output,
                        )
                        .map(|plan| {
                            (
                                plan.message_sequence(),
                                plan.client_command_acknowledge(),
                                plan.commands()
                                    .map(|command| {
                                        (command.sequence(), command.as_bytes().to_vec())
                                    })
                                    .collect::<Vec<_>>(),
                            )
                        }),
                    )
                })
                .expect("Receive")
                .expect("One datagram")
                .expect("Ordinary frame");
            if valid {
                let (sequence, acknowledge, commands) =
                    result.expect("Complete validated services");
                assert_eq!(commands, [(3, b"PrivateValid.".to_vec())]);
                assert_eq!(
                    (sequence, acknowledge, history.acknowledged_sequence()),
                    (8, 1, 0)
                );
                history
                    .commit_acknowledgement(acknowledge)
                    .expect("Host commit after complete validation");
                validated_message = sequence;
            } else {
                assert!(matches!(
                    result,
                    Err(ServiceControlError::UnsupportedService(0xfe))
                ));
                assert_eq!(output, [0xa5; 128]);
                assert_eq!(
                    (history.acknowledged_sequence(), history.pending_count()),
                    (0, 1)
                );
                assert_eq!((receiver.incoming_sequence(), validated_message), (7, 0));
                proceed.send(()).expect("Allow next server frame");
            }
        }
    }
    responder.join().expect("Responder");
    assert_eq!((validated_message, history.pending_count()), (8, 0));
    assert_eq!(channel.receive_usage().messages, 2);
}
