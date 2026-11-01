// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    thread,
    time::{Duration, Instant},
};

use cod4xr_reliable::{
    receive::{ReceiveSlot, ReceiveWindow},
    send::SendWindow,
};

use super::{receive_auxiliary, storage, wire};
use crate::{
    channel_support::{accept, acknowledged},
    support::{receive, server},
};

#[test]
fn acknowledgement_only_sends_use_consumed_base_selective_retention_and_host_window_hint() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        for (sequence, payload) in [(1, b"late"), (0, b"head")] {
            socket
                .send_to(&wire(sequence, 0, &[], i16::MAX, payload), client)
                .expect("Incoming data");
        }
        for expected in [wire(-1, 0, &[(0, 2)], 4, b""), wire(-1, 2, &[], 4, b"")] {
            let (bytes, source) = receive(&socket);
            assert_eq!(source, client);
            assert_eq!(bytes, expected);
        }
    });
    let mut ordinary_storage = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut ordinary_storage)
        .expect("Channel");
    let (mut bytes, mut slots) = storage(4);
    let mut outgoing = SendWindow::new(&mut bytes, &mut slots, 4).expect("Outgoing");
    let mut receive_bytes = [0; 3600];
    let mut receive_slots = [ReceiveSlot::default(); 3];
    let mut incoming =
        ReceiveWindow::new(&mut receive_bytes, &mut receive_slots).expect("Incoming");
    let deadline = Instant::now() + Duration::from_secs(4);
    {
        let mut parts = channel.split();
        for _ in 0..2 {
            assert_eq!(
                receive_auxiliary(&mut parts.receiver, deadline, &mut outgoing, &mut incoming)
                    .released_fragments,
                0
            );
        }
        let mut output = [0; 8];
        assert_eq!(incoming.read(&mut output[..2], 1).bytes, 2);
        assert_eq!(&output[..2], b"he");
        let sent = parts
            .sender
            .send_reliable_ack(&outgoing, incoming.acknowledgement(0), deadline)
            .expect("Selective ACK");
        assert_eq!((sent.fragment, sent.bytes), (None, 24));
        assert_eq!(incoming.read(&mut output, 2).bytes, 6);
        assert_eq!(&output[..6], b"adlate");
        let sent = parts
            .sender
            .send_reliable_ack(&outgoing, incoming.acknowledgement(0), deadline)
            .expect("Consumed ACK");
        assert_eq!((sent.fragment, sent.bytes), (None, 20));
        assert_eq!(parts.sender.next_sequence(), 1);
    }
    responder.join().expect("Responder");
    assert_eq!(
        (
            outgoing.acknowledge(),
            outgoing.next_sequence(),
            outgoing.queued_fragments()
        ),
        (0, 0, 0)
    );
    assert_eq!(
        (
            incoming.acknowledge(),
            incoming.stored_fragments(),
            incoming.buffered_bytes()
        ),
        (2, 0, 0)
    );
    let usage = channel.send_usage();
    assert_eq!(
        (
            usage.messages_reserved,
            usage.datagrams_sent,
            usage.reliable_datagrams_sent,
            usage.bytes_sent,
            usage.failed_datagrams
        ),
        (0, 2, 2, 44, 0)
    );
    assert_eq!(channel.receive_usage().auxiliary, 2);
}
