// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::wire;
use super::{
    Buffers,
    gamestate::{BODY, BUDGET, framed},
    send_when_due,
};
use crate::{
    channel_support::accept,
    support::{receive, server},
};
use cod4xr_client_core::gamestate::{
    ClientDataSlot, EntityBaselineSlot, GameStateStorage, configstrings::ConfigStringSlot,
};
use cod4xr_client_net::{ConnectionReceive, GameStateFrameError};
use cod4xr_reliable::endpoint::{DrainStatus, EndpointConfig};
use std::{
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Output {
    text: [u8; 4],
    slots: [ConfigStringSlot; 1],
    entities: [EntityBaselineSlot; 0],
    clients: [ClientDataSlot; 0],
}
impl Output {
    pub(super) fn new() -> Self {
        Self {
            text: [0xa5; 4],
            slots: [ConfigStringSlot::default()],
            entities: [],
            clients: [],
        }
    }
    pub(super) fn storage(&mut self) -> GameStateStorage<'_> {
        GameStateStorage {
            configstring_bytes: &mut self.text,
            configstring_slots: &mut self.slots,
            entity_slots: &mut self.entities,
            client_slots: &mut self.clients,
        }
    }
}

#[test]
fn prepared_frame_cancellation_keeps_identity_and_commit_releases_it_once_without_retaining_socket()
{
    let socket = server();
    let address = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        socket
            .send_to(&wire(0, 0, &[], 4, &framed(BODY)), client)
            .expect("Application frame");
        let (bytes, source) = receive(&socket);
        assert_eq!(source, client);
        assert_eq!(bytes, wire(-1, 1, &[], 4, b""));
    });
    let mut buffers = Buffers::new();
    buffers.body.resize(128, 0xa5);
    let mut connection = buffers
        .connect(address, 21, EndpointConfig::default())
        .expect("Owner");
    let mut output = Output::new();
    let before_output = output.clone();
    let before_stats = connection.stats();
    assert_eq!(
        connection
            .prepare_game_state(128, BUDGET, &mut [0; 4], output.storage())
            .expect_err("No frame yet"),
        GameStateFrameError::NoFrame
    );
    assert_eq!(output, before_output);
    assert_eq!(connection.stats(), before_stats);
    let deadline = Instant::now() + Duration::from_secs(4);
    assert!(matches!(
        connection
            .receive_until(deadline, |_| ())
            .expect("Receive")
            .expect("Datagram"),
        ConnectionReceive::Accepted(_)
    ));
    assert_eq!(
        connection.drain(&mut [0; 64], 128, 8).outcome,
        Ok(DrainStatus::Ready)
    );
    let received_at = connection.stats().reliable.last_accepted_at;
    {
        let prepared = connection
            .prepare_game_state(128, BUDGET, &mut [0; 4], output.storage())
            .expect("Typed preparation");
        assert_eq!(prepared.plan().game_state().client_index(), 3);
        assert_eq!(
            prepared
                .plan()
                .game_state()
                .configstrings()
                .next()
                .expect("Patch")
                .bytes(),
            b"new."
        );
        assert!(!format!("{prepared:?}").contains("287454020"));
        drop(prepared); // Cancel release; caller retains already checked cooked output.
    }
    assert_eq!(output.text, *b"new.");
    assert!(connection.frame().is_some());
    let stats = connection.stats();
    assert_eq!(stats.reliable.gamestate_decode_attempts, 1);
    assert_eq!(stats.reliable.gamestate_decode_failures, 0);
    assert_eq!(stats.reliable.committed_gamestates, 0);
    assert_eq!(stats.reliable.released_frames, 0);
    assert_eq!(stats.reliable.last_accepted_at, received_at);
    let plan = connection
        .prepare_game_state(128, BUDGET, &mut [0; 4], output.storage())
        .expect("Same retained frame")
        .commit();
    assert!(connection.frame().is_none());
    assert!(!connection.consume_frame());
    let stats = connection.stats();
    assert_eq!(stats.reliable.gamestate_decode_attempts, 2);
    assert_eq!(stats.reliable.gamestate_decode_failures, 0);
    assert_eq!(stats.reliable.committed_gamestates, 1);
    assert_eq!(stats.reliable.released_frames, 1);
    assert_eq!(stats.reliable.last_accepted_at, received_at);
    assert_eq!(send_when_due(&mut connection, deadline).fragment, None);
    responder.join().expect("Peer");
    drop(connection);
    buffers.body.fill(0); // Cooked output has no frame or socket lifetime.
    assert_eq!(plan.game_state().client_index(), 3);
    assert_eq!(
        plan.game_state()
            .configstrings()
            .next()
            .expect("Retained patch")
            .bytes(),
        b"new."
    );
}
