// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    thread,
    time::{Duration, Instant},
};

use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4x_transport::ChannelError;
use cod4xr_client_net::{ChannelParts, NetworkError};

use crate::{
    channel_support::{Seen, accept, acknowledged, ordinary, read},
    support::{receive, server},
};

#[test]
fn ordinary_sender_preserves_socket_qport_bits_sequence_and_empty_message() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        for (sequence, size) in [(1_u32, 0), (2, 1), (3, 1299)] {
            let (bytes, source) = receive(&socket);
            assert_eq!(source, client);
            assert_eq!(&bytes[..4], &sequence.to_le_bytes());
            assert_eq!(&bytes[4..6], &40_000_u16.to_le_bytes());
            assert_eq!(bytes.len(), 6 + size);
            assert!(bytes[6..].iter().all(|&byte| byte == b'x'));
        }
    });
    let mut storage = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut storage)
        .expect("Owned channel");
    assert_eq!(channel.context().server_challenge(), -2_147_481_234);
    assert_eq!(channel.context().summary().requested_protocol(), 21);
    let deadline = Instant::now() + Duration::from_secs(2);
    {
        let mut parts = channel.split();
        assert_eq!(parts.receiver.incoming_sequence(), 0);
        for (sequence, size) in [(1, 0), (2, 1), (3, 1299)] {
            let body = vec![b'x'; size];
            let mut message = parts.sender.begin(&body).expect("Reserve sequence");
            let sent = message
                .send_next(deadline)
                .expect("Send")
                .expect("One datagram");
            assert_eq!((sent.sequence, sent.bytes), (sequence, 6 + size));
            assert_eq!(message.send_next(deadline).expect("Complete"), None);
        }
        assert_eq!(parts.sender.next_sequence(), 4);
        assert!(!format!("{parts:?}").contains("PrivateMod"));
    }
    responder.join().expect("Responder");
    assert_eq!(channel.send_usage().messages_reserved, 3);
    assert_eq!(channel.send_usage().datagrams_sent, 3);
    assert_eq!(channel.send_usage().bytes_sent, 1318);
    assert!(!format!("{channel:?}").contains("2147481234"));
}

#[test]
fn paced_fragments_have_exact_offsets_and_terminators_and_allow_interleaved_receives() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        for (sequence, size) in [(1_u32, 1300), (2, 2600), (3, 2601)] {
            let mut start = 0;
            loop {
                let length = (size - start).min(1300);
                let (bytes, source) = receive(&socket);
                assert_eq!(source, client);
                assert_eq!(&bytes[..4], &(sequence | 0x8000_0000).to_le_bytes());
                assert_eq!(&bytes[4..6], &40_000_u16.to_le_bytes());
                assert_eq!(
                    &bytes[6..10],
                    &u32::try_from(start).expect("Offset").to_le_bytes()
                );
                assert_eq!(
                    &bytes[10..12],
                    &i16::try_from(length).expect("Length").to_le_bytes()
                );
                assert_eq!(bytes.len(), 12 + length);
                assert!(bytes[12..].iter().all(|&byte| byte == b'x'));
                if sequence == 1 && start == 0 {
                    socket
                        .send_to(&ordinary(1, b"PrivateBody"), client)
                        .expect("Receive while sending");
                }
                if length < 1300 {
                    break;
                }
                start += length;
            }
        }
    });
    let mut storage = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut storage)
        .expect("Channel");
    let deadline = Instant::now() + Duration::from_secs(2);
    {
        let ChannelParts {
            mut sender,
            mut receiver,
        } = channel.split();
        for (sequence, size) in [(1, 1300), (2, 2600), (3, 2601)] {
            let bytes = vec![b'x'; size];
            let mut message = sender.begin(&bytes).expect("Message");
            assert!(!format!("{message:?}").contains("PrivateBody"));
            assert_eq!(
                message
                    .send_next(deadline)
                    .expect("First")
                    .expect("Datagram")
                    .sequence,
                sequence
            );
            if sequence == 1 {
                assert_eq!(
                    read(&mut receiver, deadline),
                    Seen::Message(1, 0, b"PrivateBody".to_vec())
                );
            }
            while message
                .send_next(deadline)
                .expect("Next fragment")
                .is_some()
            {}
        }
    }
    responder.join().expect("Responder");
    assert_eq!(channel.send_usage().datagrams_sent, 8);
    assert_eq!(channel.receive_usage().messages, 1);
}

#[test]
fn cancellation_and_send_failure_never_reuse_sequences_or_skip_a_failed_fragment() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        let (first, source) = receive(&socket);
        assert_eq!(source, client);
        assert_eq!(&first[..4], &0x8000_0001_u32.to_le_bytes());
        let (next, source) = receive(&socket);
        assert_eq!(source, client);
        assert_eq!(&next[..4], &3_u32.to_le_bytes());
        assert_eq!(&next[6..], b"new sequence");
    });
    let mut storage = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut storage)
        .expect("Channel");
    {
        let mut parts = channel.split();
        let oversized = vec![0; MAX_MESSAGE_BYTES + 1];
        assert!(matches!(
            parts.sender.begin(&oversized),
            Err(NetworkError::Channel(ChannelError::MessageTooLarge))
        ));
        assert_eq!(parts.sender.next_sequence(), 1);
        let body = [0; 1300];
        {
            let mut cancelled = parts.sender.begin(&body).expect("First sequence");
            cancelled
                .send_next(Instant::now() + Duration::from_secs(2))
                .expect("First fragment");
        }
        {
            let mut failed = parts.sender.begin(&body).expect("Second sequence");
            assert!(matches!(
                failed.send_next(Instant::now()),
                Err(NetworkError::Timeout)
            ));
            assert!(matches!(
                failed.send_next(Instant::now() + Duration::from_secs(2)),
                Err(NetworkError::TransmissionFailed)
            ));
        }
        parts
            .sender
            .begin(b"new sequence")
            .expect("Third sequence")
            .send_next(Instant::now() + Duration::from_secs(2))
            .expect("Send");
        assert_eq!(parts.sender.next_sequence(), 4);
        assert_eq!(parts.sender.usage().failed_datagrams, 1);
    }
    responder.join().expect("Responder");
    assert_eq!(
        (
            channel.send_usage().messages_reserved,
            channel.send_usage().datagrams_sent
        ),
        (3, 2)
    );
}
