// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    thread,
    time::{Duration, Instant},
};

use cod4xr_client_net::{AuxiliaryKind, ChannelDrop, ChannelEvent, NetworkError};
use cod4xr_reliable::ReliableError;

use crate::{
    channel_support::{accept, acknowledged, ordinary},
    support::{packet, server},
};

#[test]
fn selected_qport_and_family_precede_body_parsing_without_applying_force_ack_or_window_hints() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        let body = [0xff, 0xff, 0xff, 0xff, 0, 0, 0, 0, 1, 0, 4, 0, 0, 0];
        for qport in [40_001_u16, 40_000] {
            let mut bytes = ordinary(0xffff_fff0, &qport.to_le_bytes());
            bytes.extend_from_slice(&body);
            socket.send_to(&bytes, client).expect("Wrong/correct qport");
        }
        let mut malformed = ordinary(0xffff_fff0, &40_000_u16.to_le_bytes());
        malformed.extend_from_slice(&body);
        let last_length = malformed.len() - 2;
        malformed[last_length..].copy_from_slice(&1_i16.to_le_bytes());
        socket
            .send_to(&malformed, client)
            .expect("Malformed declared suffix");
        socket
            .send_to(&packet(&body), client)
            .expect("Same bytes under another family");
    });
    let mut storage = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut storage)
        .expect("Channel");
    {
        let mut receiver = channel.split().receiver;
        let deadline = Instant::now() + Duration::from_secs(2);
        assert!(
            receiver
                .receive_until(deadline, |event| matches!(
                    event,
                    ChannelEvent::Dropped(ChannelDrop::WrongQport)
                ))
                .expect("Receive")
                .expect("Datagram")
        );
        assert!(
            receiver
                .receive_until(deadline, |event| {
                    let ChannelEvent::Auxiliary(message) = event else {
                        return false;
                    };
                    assert_eq!(message.kind(), AuxiliaryKind::Reliable);
                    let parsed = message.reliable_packet().expect("Typed native body");
                    let header = parsed.header();
                    assert_eq!(
                        (header.sequence, header.acknowledge, header.sender_window),
                        (None, 0, 4)
                    );
                    assert!(header.force_ack());
                    assert!(header.selective.is_empty());
                    parsed.payload().is_empty()
                })
                .expect("Receive")
                .expect("Datagram")
        );
        assert!(
            receiver
                .receive_until(deadline, |event| {
                    let ChannelEvent::Auxiliary(message) = event else {
                        return false;
                    };
                    matches!(
                        message.reliable_packet(),
                        Err(NetworkError::Reliable(ReliableError::LengthMismatch))
                    )
                })
                .expect("Receive")
                .expect("Datagram")
        );
        assert!(
            receiver
                .receive_until(deadline, |event| {
                    let ChannelEvent::Auxiliary(message) = event else {
                        return false;
                    };
                    assert_eq!(message.kind(), AuxiliaryKind::Connectionless);
                    matches!(message.reliable_packet(), Err(NetworkError::AuxiliaryKind))
                })
                .expect("Receive")
                .expect("Datagram")
        );
        assert_eq!(receiver.incoming_sequence(), 0);
    }
    responder.join().expect("Responder");
    let usage = channel.receive_usage();
    assert_eq!(
        (
            usage.datagrams,
            usage.dropped,
            usage.auxiliary,
            usage.messages
        ),
        (4, 1, 3, 0)
    );
    assert_eq!(channel.send_usage().messages_reserved, 0);
}
