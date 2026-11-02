// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::wire;
use super::{Buffers, send_when_due};
use crate::{
    channel_support::{accept, ordinary},
    support::{packet, receive, server},
};
use cod4xr_client_net::{
    AuxiliaryKind, ChannelDrop, ChannelEvent, ConnectionReceive, NetworkError, ReliableConnection,
    ReliableRejection,
};
use cod4xr_reliable::{
    ReliableError,
    endpoint::{DrainStatus, EndpointConfig, EndpointError},
    send::SendError,
};
use std::{
    net::UdpSocket,
    thread,
    time::{Duration, Instant},
};

fn peer(socket: UdpSocket) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let client = accept(&socket);
        let (bytes, source) = receive(&socket);
        assert_eq!(source, client);
        assert_eq!(bytes, wire(0, 0, &[], 4, b"\x06\0\0\0client"));
        let mut wrong_port = wire(0, 1, &[], 4, b"\x06\0\0\0server");
        wrong_port[4..6].copy_from_slice(&40_001_u16.to_le_bytes());
        socket.send_to(&wrong_port, client).expect("Wrong qport");
        let mut malformed = wire(0, 2, &[], 4, b"x");
        malformed[18..20].copy_from_slice(&2_i16.to_le_bytes());
        socket.send_to(&malformed, client).expect("Length mismatch");
        for acknowledge in [2, 1] {
            let mut bytes = wire(0, acknowledge, &[], 4, b"\x06\0\0\0server");
            bytes[14] = 1;
            socket.send_to(&bytes, client).expect("Combined packet");
        }
        socket
            .send_to(&ordinary(1, b"PrivateOrdinary"), client)
            .expect("Ordinary body");
        socket
            .send_to(&packet(b"print PrivateRemote"), client)
            .expect("Inert connectionless");
        let (bytes, source) = receive(&socket);
        assert_eq!(source, client);
        assert_eq!(bytes, wire(-1, 1, &[], 4, b""));
    })
}

fn next(
    connection: &mut ReliableConnection<'_>,
    deadline: Instant,
    callbacks: &mut usize,
) -> ConnectionReceive<Vec<u8>> {
    connection
        .receive_until(deadline, |event| {
            *callbacks += 1;
            let result = match event {
                ChannelEvent::Message(message) => Some(message.body().to_vec()),
                ChannelEvent::Auxiliary(message) => {
                    assert_eq!(message.kind(), AuxiliaryKind::Connectionless);
                    Some(message.body().to_vec())
                }
                ChannelEvent::Dropped(reason) => {
                    assert_eq!(reason, ChannelDrop::WrongQport);
                    Some(Vec::new())
                }
                ChannelEvent::FragmentPending => None,
            };
            result.expect("One of the scripted other events")
        })
        .expect("Receive")
        .expect("One datagram")
}

#[test]
fn owner_retains_socket_routing_windows_and_scoped_ordinary_callbacks_across_reliable_admission() {
    let socket = server();
    let address = socket.local_addr().expect("Peer");
    let responder = peer(socket);
    let mut buffers = Buffers::new();
    let mut connection = buffers
        .connect(address, 21, EndpointConfig::default())
        .expect("Owner");
    assert_eq!(connection.context().summary().requested_protocol(), 21);
    assert!(connection.server_decoder().is_ok());
    let _ = connection.enqueue(b"client").expect("Whole frame");
    assert!(matches!(
        connection.send_due(Instant::now()),
        Err(NetworkError::Timeout)
    ));
    let deadline = Instant::now() + Duration::from_secs(4);
    assert!(
        send_when_due(&mut connection, deadline)
            .fragment
            .expect("Data")
            .first_transmission
    );
    let mut callbacks = 0;
    assert!(
        matches!(next(&mut connection, deadline, &mut callbacks), ConnectionReceive::Other(data) if data.is_empty())
    );
    assert!(matches!(
        next(&mut connection, deadline, &mut callbacks),
        ConnectionReceive::Rejected(ReliableRejection::Body(ReliableError::LengthMismatch))
    ));
    assert!(matches!(
        next(&mut connection, deadline, &mut callbacks),
        ConnectionReceive::Rejected(ReliableRejection::State(EndpointError::Send(
            SendError::AckFuture
        )))
    ));
    assert_eq!(
        (
            connection.stats().endpoint.incoming_bytes,
            connection.stats().endpoint.outgoing_fragments
        ),
        (0, 1)
    );
    assert!(connection.stats().reliable.last_accepted_at.is_none());
    assert!(matches!(
        next(&mut connection, deadline, &mut callbacks),
        ConnectionReceive::Accepted(_)
    ));
    assert!(
        matches!(next(&mut connection, deadline, &mut callbacks), ConnectionReceive::Other(data) if data == b"PrivateOrdinary")
    );
    let other = next(&mut connection, deadline, &mut callbacks);
    assert_eq!(format!("{other:?}"), "Other(..)");
    assert!(matches!(other, ConnectionReceive::Other(data) if data == b"print PrivateRemote"));
    assert_eq!(callbacks, 3);
    let drain = connection.drain(&mut [0; 8], 64, 8);
    assert_eq!(
        (drain.usage.bytes, drain.outcome),
        (10, Ok(DrainStatus::Ready))
    );
    assert_eq!(connection.frame().expect("Whole body").body(), b"server");
    assert!(connection.consume_frame());
    assert!(!connection.consume_frame());
    assert_eq!(send_when_due(&mut connection, deadline).fragment, None);
    responder.join().expect("Peer");
    let stats = connection.stats();
    assert_eq!(
        (
            stats.send.datagrams_sent,
            stats.send.bytes_sent,
            stats.send.failed_datagrams
        ),
        (2, 50, 1)
    );
    assert_eq!(
        (
            stats.receive.datagrams,
            stats.receive.messages,
            stats.receive.dropped
        ),
        (6, 1, 1)
    );
    assert_eq!(
        (
            stats.reliable.accepted_datagrams,
            stats.reliable.rejected_datagrams,
            stats.reliable.drained_bytes,
            stats.reliable.released_frames
        ),
        (1, 2, 10, 1)
    );
    assert!(!format!("{connection:?}").contains("Private"));
}

#[test]
fn ordinary_sender_still_uses_the_original_socket_and_independent_channel_sequence() {
    let socket = server();
    let address = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        let (bytes, source) = receive(&socket);
        assert_eq!(source, client);
        let mut expected = 1_u32.to_le_bytes().to_vec();
        expected.extend_from_slice(&40_000_u16.to_le_bytes());
        expected.extend_from_slice(b"opaque ordinary");
        assert_eq!(bytes, expected);
    });
    let mut buffers = Buffers::new();
    let mut connection = buffers
        .connect(address, 21, EndpointConfig::default())
        .expect("Owner");
    {
        let mut sender = connection.ordinary_sender().expect("Serialized sender");
        let mut message = sender
            .begin(b"opaque ordinary")
            .expect("One ordinary message");
        assert!(
            message
                .send_next(Instant::now() + Duration::from_secs(4))
                .expect("Send")
                .is_some()
        );
    }
    responder.join().expect("Peer");
    let stats = connection.stats();
    assert_eq!(
        (
            stats.send.messages_reserved,
            stats.send.datagrams_sent,
            stats.send.reliable_datagrams_sent
        ),
        (1, 1, 0)
    );
    assert_eq!(stats.endpoint.next_outgoing_sequence, 0);
}
