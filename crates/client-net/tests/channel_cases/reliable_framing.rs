// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    thread,
    time::{Duration, Instant},
};

use cod4xr_client_net::ChannelEvent;
use cod4xr_reliable::{
    framing::FrameAssembler,
    receive::{ReceiveSlot, ReceiveWindow},
};

use crate::{
    channel_support::{accept, acknowledged},
    reliable_receive::datagram,
    support::server,
};

#[test]
fn original_udp_socket_delivers_a_complete_application_frame_after_reordered_partial_length_headers()
 {
    let source = [9, 0, 0, 0, 1, 0, 0, 0, 1, 0xfe, 0, 0x92, 0xff];
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        for (sequence, payload) in [(1, &source[1..3]), (0, &source[..1]), (2, &source[3..])] {
            socket
                .send_to(
                    &datagram(
                        sequence,
                        i16::try_from(payload.len()).expect("Small fragment"),
                        payload,
                    ),
                    client,
                )
                .expect("Partial/reordered frame");
        }
    });
    let mut ordinary_storage = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut ordinary_storage)
        .expect("Channel");
    let mut fragment_bytes = [0xa5; 3600];
    let mut fragment_slots = [ReceiveSlot::default(); 3];
    let mut window =
        ReceiveWindow::new(&mut fragment_bytes, &mut fragment_slots).expect("Receive budget");
    let mut body_storage = [0xa5; 32];
    let mut messages = FrameAssembler::new(&mut body_storage).expect("Message budget");
    let deadline = Instant::now() + Duration::from_secs(2);
    {
        let mut receiver = channel.split().receiver;
        for buffered in [0, 3, 13] {
            assert!(
                receiver
                    .receive_until(deadline, |event| {
                        let ChannelEvent::Auxiliary(message) = event else {
                            return false;
                        };
                        let packet = message.reliable_packet().expect("Exact body");
                        // No outgoing reliable data in this fixture; only this ACK is valid.
                        assert_eq!(packet.header().acknowledge, 0);
                        assert!(packet.header().selective.is_empty());
                        let _ = window.prepare(packet).expect("Receive preflight").commit();
                        true
                    })
                    .expect("Receive")
                    .expect("Datagram")
            );
            for _ in 0..16 {
                let mut output = [0; 2];
                let read = window.read(&mut output, 1);
                if read.examined == 0 {
                    break;
                }
                let admitted = messages.push(&output[..read.bytes]).expect("Frame stream");
                assert_eq!(admitted.consumed, read.bytes);
            }
            assert_eq!(messages.buffered_bytes(), buffered);
            assert_eq!(messages.frame().is_some(), buffered == 13);
        }
        assert_eq!(receiver.incoming_sequence(), 0);
    }
    assert_eq!(
        messages.frame().expect("Complete application frame").body(),
        &source[4..]
    );
    assert_eq!(
        (
            window.acknowledge(),
            window.stored_fragments(),
            window.buffered_bytes()
        ),
        (3, 0, 0)
    );
    assert_eq!(channel.receive_usage().auxiliary, 3);
    assert_eq!(channel.send_usage().messages_reserved, 0);
    responder.join().expect("Responder");
}
