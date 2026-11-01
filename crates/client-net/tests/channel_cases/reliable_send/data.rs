// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    thread,
    time::{Duration, Instant},
};

use cod4xr_client_net::NetworkError;
use cod4xr_reliable::{
    ReliablePacket,
    receive::{ReceiveSlot, ReceiveWindow},
    send::{SendError, SendWindow},
};

use super::{receive_auxiliary, storage, wire};
use crate::{
    channel_support::{accept, acknowledged},
    support::{receive, server},
};

#[test]
fn expired_send_never_marks_data_and_actual_retries_preserve_original_socket_sequence_and_payload()
{
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        for _ in 0..2 {
            let (bytes, source) = receive(&socket);
            assert_eq!(source, client);
            assert_eq!(bytes, wire(0, 0, &[], 4, b"PrivateRPC"));
        }
        socket
            .send_to(&wire(-1, 1, &[], 4, b""), client)
            .expect("Consumed ACK");
    });
    let mut ordinary_storage = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut ordinary_storage)
        .expect("Channel");
    let (mut bytes, mut slots) = storage(4);
    let mut outgoing = SendWindow::new(&mut bytes, &mut slots, 4).expect("Outgoing");
    let _ = outgoing.enqueue(b"PrivateRPC").expect("Queue exact bytes");
    let mut receive_bytes = [0; 3600];
    let mut receive_slots = [ReceiveSlot::default(); 3];
    let mut incoming =
        ReceiveWindow::new(&mut receive_bytes, &mut receive_slots).expect("Incoming");
    let deadline = Instant::now() + Duration::from_secs(4);
    {
        let mut parts = channel.split();
        let tx = outgoing.prepare_transmission(0).expect("Prepare");
        assert!(matches!(
            parts
                .sender
                .send_reliable_fragment(tx, incoming.acknowledgement(0), Instant::now()),
            Err(NetworkError::Timeout)
        ));
        let ack = wire(-1, 1, &[], 4, b"");
        let ack = ReliablePacket::parse(&ack[6..]).expect("Independent ACK");
        assert_eq!(
            outgoing
                .prepare_acknowledgement(ack)
                .expect_err("Never sent"),
            SendError::AckUnsent
        );
        for first in [true, false] {
            let tx = outgoing
                .prepare_transmission(0)
                .expect("Same original fragment");
            let sent = parts
                .sender
                .send_reliable_fragment(tx, incoming.acknowledgement(0), deadline)
                .expect("Actual OS send");
            assert_eq!(sent.bytes, 30);
            let fragment = sent.fragment.expect("Data admission");
            assert_eq!(
                (
                    fragment.sequence,
                    fragment.bytes,
                    fragment.first_transmission
                ),
                (0, 10, first)
            );
        }
        assert_eq!(
            receive_auxiliary(&mut parts.receiver, deadline, &mut outgoing, &mut incoming)
                .released_fragments,
            1
        );
        assert_eq!(parts.sender.next_sequence(), 1);
        assert!(!format!("{:?}", parts.sender).contains("PrivateRPC"));
    }
    responder.join().expect("Responder");
    assert_eq!(
        (
            outgoing.acknowledge(),
            outgoing.queued_fragments(),
            outgoing.buffered_bytes()
        ),
        (1, 0, 0)
    );
    let usage = channel.send_usage();
    assert_eq!(
        (
            usage.messages_reserved,
            usage.datagrams_sent,
            usage.reliable_datagrams_sent,
            usage.bytes_sent
        ),
        (0, 2, 2, 60)
    );
    assert_eq!(
        (usage.failed_datagrams, usage.reliable_failed_datagrams),
        (1, 1)
    );
    assert_eq!(channel.receive_usage().auxiliary, 1);
}
