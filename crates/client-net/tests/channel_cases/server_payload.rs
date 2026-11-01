// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    net::SocketAddr,
    thread,
    time::{Duration, Instant},
};

use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
};
use cod4xr_client_net::{AcknowledgedSocket, ChannelEvent, ConnectOutcome, NetworkError};

use crate::{
    channel_support::{accept, ordinary},
    support::{budget, challenged, server, settings},
};

fn acknowledged_for(peer: SocketAddr, protocol: i32) -> AcknowledgedSocket {
    let mut selected = settings();
    selected.protocol = protocol;
    match challenged(peer)
        .into_pending(selected, budget())
        .expect("Pending")
        .run()
        .outcome
        .expect("Terminal outcome")
    {
        ConnectOutcome::Acknowledged(socket) => Some(socket),
        ConnectOutcome::Refused(_) => None,
    }
    .expect("Acknowledged")
}

#[test]
fn received_payload_uses_original_cookie_and_explicit_connect_profile_on_the_same_socket() {
    for protocol in [6, 21] {
        let socket = server();
        let peer = socket.local_addr().expect("Peer");
        let responder = thread::spawn(move || {
            let client = accept(&socket);
            // NOP/EOF Huffman e1 00 XOR original cookie low byte 6e and sequence 7.
            socket
                .send_to(&ordinary(7, &[0, 0, 0, 0, 0x88, 0x69]), client)
                .expect("Independent server frame");
        });
        let mut reassembly = [0; 1300];
        let mut channel = acknowledged_for(peer, protocol)
            .into_channel(&mut reassembly)
            .expect("Channel");
        let decoder = channel.server_decoder().expect("Original profile decoder");
        assert!(!format!("{decoder:?}").contains("2147481234"));
        let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let history = LegacyClientCommands::new(&mut storage).expect("History");
        let mut work = [0xa5; 8];
        let mut output = [0xa5; 16];
        let received = {
            let mut receiver = channel.split().receiver;
            receiver
                .receive_until(
                    Instant::now() + Duration::from_secs(2),
                    |event| match event {
                        ChannelEvent::Message(message) => {
                            let payload = decoder
                                .decode(
                                    message.sequence(),
                                    message.body(),
                                    &history,
                                    &mut work,
                                    &mut output,
                                )
                                .expect("Decoded server bytes");
                            Some((
                                payload.profile(),
                                payload.sequence(),
                                payload.client_command_acknowledge(),
                                payload.body().to_vec(),
                            ))
                        }
                        _ => None,
                    },
                )
                .expect("Receive")
                .expect("One datagram")
                .expect("Ordinary message")
        };
        responder.join().expect("Responder");
        assert_eq!(
            received,
            (
                BaselineProtocol::from_number(protocol).expect("Explicit profile"),
                7,
                0,
                vec![0, 7, 6, 6]
            )
        );
        assert_eq!(history.acknowledged_sequence(), 0);
        assert_eq!(channel.receive_usage().messages, 1);
    }
}

#[test]
fn unreviewed_original_protocol_does_not_gain_a_decoder_from_the_challenge_hint() {
    for protocol in [18, 42] {
        let socket = server();
        let peer = socket.local_addr().expect("Peer");
        let responder = thread::spawn(move || {
            accept(&socket);
        });
        let mut reassembly = [0; 1300];
        let channel = acknowledged_for(peer, protocol)
            .into_channel(&mut reassembly)
            .expect("Opaque ordinary channel");
        responder.join().expect("Responder");
        assert!(matches!(
            channel.server_decoder(),
            Err(NetworkError::UnsupportedProtocol)
        ));
        assert_eq!(channel.context().summary().requested_protocol(), protocol);
        assert_eq!(channel.receive_usage().datagrams, 0);
    }
}
