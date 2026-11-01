// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    thread,
    time::{Duration, Instant},
};

use cod4xr_client_net::{ChannelEvent, NetworkError};
use cod4xr_reliable::{
    ReliableError,
    receive::{ReceiveDisposition, ReceiveSlot, ReceiveWindow},
};

use crate::{
    channel_support::{accept, acknowledged, ordinary},
    support::server,
};

fn datagram(sequence: u32, declared: i16, payload: &[u8]) -> Vec<u8> {
    let mut bytes = ordinary(0xffff_fff0, &40_000_u16.to_le_bytes());
    bytes.extend_from_slice(&sequence.to_le_bytes());
    bytes.extend_from_slice(&0_i32.to_le_bytes());
    bytes.extend_from_slice(&[0, 0]); // flags and SACK count
    bytes.extend_from_slice(&4_i16.to_le_bytes());
    bytes.extend_from_slice(&declared.to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}

#[test]
fn checked_udp_bodies_reassemble_reordered_bytes_without_accepting_malformed_or_duplicate_data() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        for (sequence, declared, payload) in [
            (1, 3, b"DEF".as_slice()),
            (0, 4, b"abc".as_slice()),
            (0, 3, b"abc".as_slice()),
            (0, 3, b"BAD".as_slice()),
        ] {
            socket
                .send_to(&datagram(sequence, declared, payload), client)
                .expect("Reliable body");
        }
    });
    let mut ordinary_storage = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut ordinary_storage)
        .expect("Channel");
    let mut bytes = [0xa5; 3600];
    let mut slots = [ReceiveSlot::default(); 3];
    let mut window = ReceiveWindow::new(&mut bytes, &mut slots).expect("Receive budget");
    let deadline = Instant::now() + Duration::from_secs(2);
    {
        let mut receiver = channel.split().receiver;
        let expected = [
            Some(ReceiveDisposition::Stored {
                sequence: 1,
                bytes: 3,
            }),
            None,
            Some(ReceiveDisposition::Stored {
                sequence: 0,
                bytes: 3,
            }),
            Some(ReceiveDisposition::Duplicate { sequence: 0 }),
        ];
        for disposition in expected {
            let actual = receiver
                .receive_until(deadline, |event| {
                    let ChannelEvent::Auxiliary(message) = event else {
                        return None;
                    };
                    let packet = match message.reliable_packet() {
                        Ok(packet) => packet,
                        Err(error) => {
                            assert!(matches!(
                                error,
                                NetworkError::Reliable(ReliableError::LengthMismatch)
                            ));
                            return None;
                        }
                    };
                    // This fixture has no outgoing reliable data: only ACK zero/no SACKs
                    // is admissible. General outgoing-window admission remains separate.
                    assert_eq!(packet.header().acknowledge, 0);
                    assert!(packet.header().selective.is_empty());
                    Some(window.prepare(packet).expect("Receive preflight").commit())
                })
                .expect("Receive")
                .expect("Datagram");
            assert_eq!(actual, disposition);
        }
        assert_eq!(receiver.incoming_sequence(), 0);
    }
    assert_eq!(
        (
            window.stored_fragments(),
            window.buffered_bytes(),
            window.acknowledge()
        ),
        (2, 6, 0)
    );
    let mut output = [0xa5; 8];
    assert_eq!(window.read(&mut output[..2], 1).bytes, 2);
    assert_eq!(&output[..2], b"ab");
    assert_eq!(window.acknowledge(), 0);
    assert_eq!(window.read(&mut output, 2).bytes, 4);
    assert_eq!(&output[..4], b"cDEF");
    assert_eq!(&output[4..], &[0xa5; 4]);
    assert_eq!(window.acknowledge(), 2);
    assert_eq!((window.stored_fragments(), window.buffered_bytes()), (0, 0));
    assert_eq!(channel.receive_usage().auxiliary, 4);
    assert_eq!(channel.send_usage().messages_reserved, 0);
    responder.join().expect("Responder");
}
