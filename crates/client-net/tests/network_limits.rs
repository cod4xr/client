// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded network work includes queued phase delays and informational traffic.
#![allow(
    clippy::expect_used,
    reason = "Finite loopback fixtures with joined threads."
)]
mod support;
use std::{sync::mpsc, thread, time::Duration};

use cod4xr_client_core::{
    connect::ConnectFailure,
    handshake::{ChallengeBudget, ChallengeFailure},
};
use cod4xr_client_net::{ConnectOutcome, NetworkError, challenge, datagram};
use support::{budget, challenged, packet, receive, reply_challenge, server, settings};

#[test]
fn last_receive_slot_allows_acknowledgement_but_not_another_notice() {
    for accepted in [false, true] {
        let socket = server();
        let peer = socket.local_addr().expect("Peer");
        let responder = thread::spawn(move || {
            let client = reply_challenge(&socket);
            receive(&socket);
            socket
                .send_to(&packet(b"print\none"), client)
                .expect("First notice");
            let body = if accepted {
                b"connectResponse".as_slice()
            } else {
                b"print\ntwo".as_slice()
            };
            socket.send_to(&packet(body), client).expect("Last slot");
        });
        let mut limits = budget();
        limits.max_datagrams = 2;
        let run = challenged(peer)
            .into_pending(settings(), limits)
            .expect("Pending")
            .run();
        responder.join().expect("Responder");
        assert_eq!(run.usage.received, 2);
        if accepted {
            assert!(matches!(run.outcome, Ok(ConnectOutcome::Acknowledged(_))));
            assert_eq!(run.usage.notices, 1);
        } else {
            assert!(matches!(
                run.outcome,
                Err(NetworkError::Connect(ConnectFailure::ReceiveBudget))
            ));
            assert_eq!(run.usage.notices, 2);
        }
    }
}

#[test]
fn waiting_to_run_does_not_restart_the_pending_deadline_or_send_a_request() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let (release, wait) = mpsc::channel();
    let responder = thread::spawn(move || {
        reply_challenge(&socket);
        wait.recv_timeout(Duration::from_secs(4))
            .expect("Client completed");
        socket.set_nonblocking(true).expect("Check queued work");
        let error = socket
            .recv_from(&mut [0; 128])
            .expect_err("No connect after queued expiry");
        assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock);
    });
    let mut limits = budget();
    limits.timeout = Duration::from_millis(20);
    let pending = challenged(peer)
        .into_pending(settings(), limits)
        .expect("Pending");
    thread::sleep(Duration::from_millis(40));
    let run = pending.run();
    release.send(()).expect("Release peer");
    responder.join().expect("Responder");
    assert!(matches!(
        run.outcome,
        Err(NetworkError::Connect(ConnectFailure::Expired))
    ));
    assert_eq!(run.usage.attempts, 0);
}

#[test]
fn silent_peer_expires_after_last_send_and_preserves_failure_counters() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let (release, wait) = mpsc::channel();
    let responder = thread::spawn(move || {
        reply_challenge(&socket);
        receive(&socket);
        wait.recv_timeout(Duration::from_secs(4))
            .expect("Client expired");
    });
    let mut limits = budget();
    limits.timeout = Duration::from_millis(75);
    limits.max_attempts = 1;
    let run = challenged(peer)
        .into_pending(settings(), limits)
        .expect("Pending")
        .run();
    release.send(()).expect("Release peer");
    responder.join().expect("Responder");
    assert!(matches!(
        run.outcome,
        Err(NetworkError::Connect(ConnectFailure::Expired))
    ));
    assert_eq!((run.usage.attempts, run.usage.received), (1, 0));
}

#[test]
fn invalid_budgets_and_endpoints_fail_before_admitting_network_work() {
    for address in [
        "0.0.0.0:28960",
        "224.0.0.1:28960",
        "127.0.0.1:0",
        "[::]:28960",
        "[ff02::1]:28960",
    ] {
        assert!(matches!(
            datagram::connect(address.parse().expect("Endpoint")),
            Err(NetworkError::InvalidPeer)
        ));
    }
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let result = challenge(
        peer,
        ChallengeBudget {
            max_datagrams: 0,
            ..ChallengeBudget::default()
        },
    );
    assert!(matches!(
        result.outcome,
        Err(NetworkError::Challenge(ChallengeFailure::InvalidBudget))
    ));
    assert_eq!(result.usage.attempts, 0);
    socket.set_nonblocking(true).expect("Inspect no work");
    assert_eq!(
        socket.recv_from(&mut [0; 128]).expect_err("No send").kind(),
        std::io::ErrorKind::WouldBlock
    );
}
