// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    thread,
    time::{Duration, Instant},
};

use cod4x_transport::ReceiveDrop;
use cod4xr_client_net::{AuxiliaryKind, ChannelDrop};

use crate::{
    channel_support::{Seen, accept, acknowledged, fragment, ordinary, read},
    support::{packet, server},
};

#[test]
fn ordinary_receive_filters_stale_sequences_and_malformed_fragments_without_byte_disclosure() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        let mut bad_length = fragment(3, 0, b"bad");
        bad_length[8..10].copy_from_slice(&4_i16.to_le_bytes());
        for bytes in [
            ordinary(2, b"PrivateBody\xff"),
            ordinary(2, b"duplicate"),
            ordinary(1, b"stale"),
            bad_length,
            ordinary(4, b"next"),
        ] {
            socket.send_to(&bytes, client).expect("Sequenced datagram");
        }
    });
    let mut storage = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut storage)
        .expect("Channel");
    let deadline = Instant::now() + Duration::from_secs(2);
    {
        let mut parts = channel.split();
        assert_eq!(
            read(&mut parts.receiver, deadline),
            Seen::Message(2, 1, b"PrivateBody\xff".to_vec())
        );
        for _ in 0..2 {
            assert_eq!(
                read(&mut parts.receiver, deadline),
                Seen::Dropped(ChannelDrop::State(ReceiveDrop::StaleSequence))
            );
        }
        assert_eq!(
            read(&mut parts.receiver, deadline),
            Seen::Dropped(ChannelDrop::Malformed)
        );
        assert_eq!(
            read(&mut parts.receiver, deadline),
            Seen::Message(4, 1, b"next".to_vec())
        );
        assert_eq!(parts.receiver.incoming_sequence(), 4);
    }
    responder.join().expect("Responder");
    let usage = channel.receive_usage();
    assert_eq!((usage.datagrams, usage.messages, usage.dropped), (5, 2, 3));
}

#[test]
fn fragments_preserve_partial_bytes_under_reordering_duplicates_and_older_sequences() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        for bytes in [
            fragment(2, 1300, b"tail"),
            fragment(2, 0, &[b'x'; 1300]),
            fragment(2, 0, &[b'x'; 1300]),
            fragment(1, 0, &[b'y'; 1300]),
            fragment(2, 1300, b"tail"),
        ] {
            socket.send_to(&bytes, client).expect("Fragment");
        }
    });
    let mut storage = [0; 2600];
    let mut channel = acknowledged(peer)
        .into_channel(&mut storage)
        .expect("Channel");
    let deadline = Instant::now() + Duration::from_secs(2);
    {
        let mut receiver = channel.split().receiver;
        assert_eq!(
            read(&mut receiver, deadline),
            Seen::Dropped(ChannelDrop::State(ReceiveDrop::OutOfOrderFragment))
        );
        assert_eq!(receiver.partial_bytes(), 0);
        assert_eq!(read(&mut receiver, deadline), Seen::Pending);
        assert_eq!(receiver.partial_bytes(), 1300);
        assert_eq!(
            read(&mut receiver, deadline),
            Seen::Dropped(ChannelDrop::State(ReceiveDrop::OutOfOrderFragment))
        );
        assert_eq!(
            read(&mut receiver, deadline),
            Seen::Dropped(ChannelDrop::State(ReceiveDrop::StaleSequence))
        );
        assert_eq!(receiver.partial_bytes(), 1300);
        let mut expected = vec![b'x'; 1300];
        expected.extend_from_slice(b"tail");
        assert_eq!(read(&mut receiver, deadline), Seen::Message(2, 1, expected));
        assert_eq!(
            (receiver.incoming_sequence(), receiver.partial_bytes()),
            (2, 0)
        );
        assert_eq!(receiver.usage().pending_fragments, 1);
    }
    responder.join().expect("Responder");
}

#[test]
fn exactly_full_fragment_does_not_deliver_until_zero_length_terminator() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        for bytes in [fragment(1, 0, &[b'x'; 1300]), fragment(1, 1300, b"")] {
            socket.send_to(&bytes, client).expect("Exact multiple");
        }
    });
    let mut storage = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut storage)
        .expect("Channel");
    let deadline = Instant::now() + Duration::from_secs(2);
    {
        let mut receiver = channel.split().receiver;
        assert_eq!(read(&mut receiver, deadline), Seen::Pending);
        assert_eq!(receiver.incoming_sequence(), 0);
        assert_eq!(
            read(&mut receiver, deadline),
            Seen::Message(1, 0, vec![b'x'; 1300])
        );
    }
    responder.join().expect("Responder");
    assert_eq!(channel.receive_usage().messages, 1);
}

#[test]
fn auxiliary_markers_precede_fragment_flag_and_reliable_qport_is_checked() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        let update = ordinary(0xffff_fffe, b"PrivateUpdate");
        let mut reliable = ordinary(0xffff_fff0, &40_000_u16.to_le_bytes());
        reliable.extend_from_slice(b"PrivateReliable");
        let mut wrong_qport = reliable.clone();
        wrong_qport[4..6].copy_from_slice(&40_001_u16.to_le_bytes());
        for bytes in [
            packet(b"print\nPrivateNotice"),
            update,
            reliable,
            wrong_qport,
            vec![0xf0, 0xff, 0xff, 0xff, 1],
            ordinary(1, b"PrivateBody"),
        ] {
            socket
                .send_to(&bytes, client)
                .expect("Auxiliary or ordinary packet");
        }
    });
    let mut storage = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut storage)
        .expect("Channel");
    let deadline = Instant::now() + Duration::from_secs(2);
    {
        let mut receiver = channel.split().receiver;
        for (kind, body) in [
            (
                AuxiliaryKind::Connectionless,
                b"print\nPrivateNotice".as_slice(),
            ),
            (AuxiliaryKind::UpdateProxy, b"PrivateUpdate"),
            (AuxiliaryKind::Reliable, b"PrivateReliable"),
        ] {
            assert_eq!(
                read(&mut receiver, deadline),
                Seen::Auxiliary(kind, body.to_vec())
            );
            assert_eq!(receiver.incoming_sequence(), 0);
        }
        assert_eq!(
            read(&mut receiver, deadline),
            Seen::Dropped(ChannelDrop::WrongQport)
        );
        assert_eq!(
            read(&mut receiver, deadline),
            Seen::Dropped(ChannelDrop::Malformed)
        );
        assert_eq!(
            read(&mut receiver, deadline),
            Seen::Message(1, 0, b"PrivateBody".to_vec())
        );
    }
    responder.join().expect("Responder");
    let usage = channel.receive_usage();
    assert_eq!(
        (
            usage.datagrams,
            usage.auxiliary,
            usage.dropped,
            usage.messages
        ),
        (6, 3, 2, 1)
    );
}
