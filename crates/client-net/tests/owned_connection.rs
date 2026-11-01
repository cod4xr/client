// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent UDP services validate socket lifetime and request stability.
#![allow(
    clippy::expect_used,
    reason = "Bounded loopback fixtures with joined threads."
)]
mod support;
use std::{
    net::UdpSocket,
    thread,
    time::{Duration, Instant},
};

use cod4xr_client_core::connect::ConnectReplyKind;
use cod4xr_client_net::{ConnectOutcome, datagram};
use support::{budget, challenged, packet, receive, reply_challenge, server, settings};

#[test]
fn one_socket_survives_challenge_connect_retries_and_explicit_channel_handoff() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = reply_challenge(&socket);
        let (first, source) = receive(&socket);
        assert_eq!(source, client);
        assert!(first.starts_with(b"\xff\xff\xff\xffconnect \"\\"));
        assert_eq!(first.last(), Some(&b'"'));
        let info = std::str::from_utf8(&first[13..first.len() - 1]).expect("ASCII fixture");
        let fields: Vec<_> = info.split('\\').skip(1).collect();
        assert_eq!(fields.len() % 2, 0);
        for (key, value) in [
            ("protocol", "21"),
            ("challenge", "-2147481234"),
            ("qport", "-25536"),
            ("xver", "21.8"),
            ("name", "PrivatePlayer"),
            ("password", "PrivatePassword"),
        ] {
            assert!(fields.chunks_exact(2).any(|pair| pair == [key, value]));
        }
        assert!(!fields.contains(&"pbguid"));
        let (second, source) = receive(&socket);
        assert_eq!(source, client);
        assert_eq!(first, second);
        socket
            .send_to(&packet(b"print\nPrivateNotice"), client)
            .expect("Notice");
        socket
            .send_to(&packet(b"connectResponse mods/PrivateMod"), client)
            .expect("Acknowledgement");
        let (marker, source) = receive(&socket);
        assert_eq!(source, client);
        assert_eq!(marker, b"test-only-owner-handoff");
        client
    });
    let challenged = challenged(peer);
    assert!(!format!("{challenged:?}").contains("2147481234"));
    let pending = challenged
        .into_pending(settings(), budget())
        .expect("Owned pending phase");
    assert!(!format!("{pending:?}").contains("PrivatePassword"));
    let run = pending.run();
    assert_eq!(
        (run.usage.attempts, run.usage.received, run.usage.notices),
        (2, 2, 1)
    );
    assert!(!format!("{run:?}").contains("PrivateMod"));
    let acknowledged = match run.outcome.expect("Connect terminal result") {
        ConnectOutcome::Acknowledged(socket) => Some(socket),
        ConnectOutcome::Refused(_) => None,
    }
    .expect("Acknowledged transport");
    let summary = acknowledged.summary();
    assert_eq!(summary.peer(), peer);
    assert_eq!(summary.requested_protocol(), 21);
    assert_eq!(summary.qport(), 40_000);
    assert_eq!(summary.kind(), ConnectReplyKind::Accepted);
    assert_eq!(summary.payload(), b"mods/PrivateMod");
    assert_eq!(summary.usage(), run.usage);
    assert!(summary.elapsed() < Duration::from_secs(2));
    let (socket, context) = acknowledged.into_parts();
    assert_eq!(socket.peer_addr().expect("Retained peer"), peer);
    assert_eq!(context.summary().payload(), b"mods/PrivateMod");
    assert_eq!(context.server_challenge(), -2_147_481_234);
    assert!(!format!("{context:?}").contains("2147481234"));
    datagram::send(
        &socket,
        b"test-only-owner-handoff",
        Instant::now() + Duration::from_secs(2),
    )
    .expect("Send through transferred original socket");
    assert_eq!(
        socket.local_addr().expect("Local endpoint"),
        responder.join().expect("Responder")
    );
}

#[test]
fn refusal_is_owned_inert_data_and_closes_the_pending_socket() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = reply_challenge(&socket);
        let (_, source) = receive(&socket);
        assert_eq!(source, client);
        socket
            .send_to(&packet(b"print\nPrivateNotice"), client)
            .expect("Notice");
        socket
            .send_to(&packet(b"error\n{ steamonly PrivateReason }"), client)
            .expect("Refusal");
        client
    });
    let run = challenged(peer)
        .into_pending(settings(), budget())
        .expect("Pending")
        .run();
    assert_eq!(
        (run.usage.attempts, run.usage.received, run.usage.notices),
        (1, 2, 1)
    );
    assert!(!format!("{run:?}").contains("PrivateReason"));
    let summary = match run.outcome.expect("Refusal outcome") {
        ConnectOutcome::Refused(summary) => Some(summary),
        ConnectOutcome::Acknowledged(_) => None,
    }
    .expect("Refusal must not grant channel ownership");
    assert_eq!(summary.kind(), ConnectReplyKind::Refused);
    assert_eq!(summary.payload(), b"{ steamonly PrivateReason }");
    let client = responder.join().expect("Responder");
    let _reclaimed = UdpSocket::bind(client).expect("Refused client socket was closed");
}

#[test]
fn oversized_datagram_cannot_become_an_accepted_or_refused_prefix() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = reply_challenge(&socket);
        receive(&socket);
        let mut oversized = packet(b"error\n");
        oversized.resize(10_000, b'x');
        socket
            .send_to(&oversized, client)
            .expect("Oversized refusal");
        socket
            .send_to(&packet(b"connectResponse"), client)
            .expect("Actual acknowledgement");
    });
    let run = challenged(peer)
        .into_pending(settings(), budget())
        .expect("Pending")
        .run();
    responder.join().expect("Responder");
    assert_eq!((run.usage.received, run.usage.rejected), (2, 1));
    assert!(matches!(run.outcome, Ok(ConnectOutcome::Acknowledged(_))));
}

#[test]
fn another_endpoint_cannot_acknowledge_the_selected_peer() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = reply_challenge(&socket);
        receive(&socket);
        server()
            .send_to(&packet(b"connectResponse mods/OtherPeer"), client)
            .expect("Wrong peer");
        socket
            .send_to(&packet(b"connectResponse mods/SelectedPeer"), client)
            .expect("Selected peer");
    });
    let run = challenged(peer)
        .into_pending(settings(), budget())
        .expect("Pending")
        .run();
    responder.join().expect("Responder");
    assert_eq!((run.usage.received, run.usage.rejected), (1, 0));
    let payload = match run.outcome.expect("Outcome") {
        ConnectOutcome::Acknowledged(socket) => socket.summary().payload().to_vec(),
        ConnectOutcome::Refused(_) => Vec::new(),
    };
    assert_eq!(payload, b"mods/SelectedPeer");
}
