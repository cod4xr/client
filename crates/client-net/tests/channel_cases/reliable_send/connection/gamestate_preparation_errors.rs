// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::wire;
use super::{
    Buffers,
    gamestate::{BODY, BUDGET, framed},
    gamestate_preparation::Output,
    send_when_due,
};
use crate::{
    channel_support::accept,
    support::{receive, server},
};
use cod4xr_client_core::gamestate::ReliableGameStateError;
use cod4xr_client_net::{ConnectionReceive, GameStateFrameError};
use cod4xr_reliable::endpoint::{DrainStatus, EndpointConfig};
use std::{
    thread,
    time::{Duration, Instant},
};

#[test]
fn failed_typed_decode_preserves_ready_frame_and_only_explicit_commit_charges_admission() {
    let socket = server();
    let address = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        let mut bad = BODY.to_vec();
        bad.push(0);
        socket
            .send_to(&wire(0, 0, &[], 4, &framed(&bad)), client)
            .expect("Bad application body");
        socket
            .send_to(&wire(1, 0, &[], 4, &framed(BODY)), client)
            .expect("Next valid body");
        let (bytes, source) = receive(&socket);
        assert_eq!(source, client);
        assert_eq!(bytes, wire(-1, 2, &[], 4, b""));
    });
    let mut buffers = Buffers::new();
    buffers.body.resize(128, 0xa5);
    let mut connection = buffers
        .connect(address, 21, EndpointConfig::default())
        .expect("Owner");
    let deadline = Instant::now() + Duration::from_secs(4);
    for _ in 0..2 {
        assert!(matches!(
            connection
                .receive_until(deadline, |_| ())
                .expect("Receive")
                .expect("Packet"),
            ConnectionReceive::Accepted(_)
        ));
    }
    assert_eq!(
        connection.drain(&mut [0; 64], 128, 8).outcome,
        Ok(DrainStatus::Ready)
    );
    let mut output = Output::new();
    let before = output.clone();
    assert_eq!(
        connection
            .prepare_game_state(128, BUDGET, &mut [0; 4], output.storage())
            .expect_err("Late invalid byte"),
        GameStateFrameError::Decode(ReliableGameStateError::TrailingBytes)
    );
    assert_eq!(output, before);
    assert!(connection.frame().is_some());
    let stats = connection.stats();
    assert_eq!(stats.reliable.gamestate_decode_attempts, 1);
    assert_eq!(stats.reliable.gamestate_decode_failures, 1);
    assert_eq!(stats.reliable.committed_gamestates, 0);
    assert_eq!(stats.reliable.released_frames, 0);
    assert!(connection.consume_frame()); // Explicit discard, not a typed commit.
    assert_eq!(
        connection.drain(&mut [0; 64], 128, 8).outcome,
        Ok(DrainStatus::Ready)
    );
    let plan = connection
        .prepare_game_state(128, BUDGET, &mut [0; 4], output.storage())
        .expect("Next valid body")
        .commit();
    assert_eq!(plan.game_state().config_data_sequence(), 9);
    let stats = connection.stats();
    assert_eq!(stats.reliable.gamestate_decode_attempts, 2);
    assert_eq!(stats.reliable.gamestate_decode_failures, 1);
    assert_eq!(stats.reliable.committed_gamestates, 1);
    assert_eq!(stats.reliable.released_frames, 2);
    assert_eq!(send_when_due(&mut connection, deadline).fragment, None);
    responder.join().expect("Peer");
}

#[test]
fn terminal_framing_failure_prevents_decode_and_preserves_output_and_work_counters() {
    let socket = server();
    let address = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        socket
            .send_to(&wire(0, 0, &[], 4, &[0xff; 4]), client)
            .expect("Negative frame length");
    });
    let mut buffers = Buffers::new();
    let mut connection = buffers
        .connect(address, 21, EndpointConfig::default())
        .expect("Owner");
    assert!(matches!(
        connection
            .receive_until(Instant::now() + Duration::from_secs(4), |_| ())
            .expect("Receive")
            .expect("Packet"),
        ConnectionReceive::Accepted(_)
    ));
    assert!(connection.drain(&mut [0; 4], 4, 1).outcome.is_err());
    let mut output = Output::new();
    let before = output.clone();
    let stats = connection.stats();
    assert_eq!(
        connection
            .prepare_game_state(128, BUDGET, &mut [0; 4], output.storage())
            .expect_err("Terminal state"),
        GameStateFrameError::Failed
    );
    assert_eq!(output, before);
    assert_eq!(connection.stats(), stats);
    assert_eq!(stats.reliable.gamestate_decode_attempts, 0);
    responder.join().expect("Peer");
}
