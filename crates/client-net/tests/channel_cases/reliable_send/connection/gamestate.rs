// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::wire;
use super::{Buffers, send_when_due};
use crate::{
    channel_support::accept,
    support::{receive, server},
};
use cod4xr_client_core::{
    BaselineProtocol,
    gamestate::{
        GameStateBudget, GameStateStorage, ReliableGameStateError, ReliableGameStatePlan,
        configstrings::ConfigStringSlot,
    },
};
use cod4xr_client_net::ConnectionReceive;
use cod4xr_reliable::endpoint::{DrainStatus, EndpointConfig};
use std::{
    thread,
    time::{Duration, Instant},
};

pub(super) const BODY: &[u8] = &[
    1, 0, 0, 0, 1, // Application command and embedded opcode.
    7, 0, 0, 0, // Server command baseline.
    2, 1, 0, 0, 0, 0, 0, 0, 0, b'n', b'e', b'w', b'%', 0, // One patch.
    7, 9, 0, 0, 0, 3, 0, 0, 0, 0x44, 0x33, 0x22, 0x11, // EOF/trailer.
    0, 0, 0, 0, // Reserved final word.
];
pub(super) const BUDGET: GameStateBudget = GameStateBudget {
    max_services: 2,
    max_configstrings: 1,
    max_text_bytes: 4,
    max_entities: 0,
    max_client_records: 0,
};

pub(super) fn framed(body: &[u8]) -> Vec<u8> {
    let mut bytes = i32::try_from(body.len())
        .expect("Small frame")
        .to_le_bytes()
        .to_vec();
    bytes.extend_from_slice(body);
    bytes
}

#[test]
fn reordered_udp_frames_require_whole_typed_admission_before_releasing_or_publishing_gamestate() {
    let socket = server();
    let address = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        let mut bad = BODY.to_vec();
        bad.push(0); // Valid embedded gamestate with a late invalid extra byte.
        socket
            .send_to(&wire(1, 0, &[], 4, &framed(BODY)), client)
            .expect("Reordered later frame");
        socket
            .send_to(&wire(0, 0, &[], 4, &framed(&bad)), client)
            .expect("Earlier malformed application frame");
        let (bytes, source) = receive(&socket);
        assert_eq!(source, client);
        assert_eq!(bytes, wire(-1, 2, &[], 4, b"")); // Byte receipt is separate from application validity.
    });
    let mut buffers = Buffers::new();
    buffers.body.resize(128, 0xa5);
    let mut connection = buffers
        .connect(address, 21, EndpointConfig::default())
        .expect("Owned socket");
    let deadline = Instant::now() + Duration::from_secs(4);
    let mut callbacks = 0;
    for _ in 0..2 {
        assert!(matches!(
            connection
                .receive_until(deadline, |_| {
                    callbacks += 1;
                })
                .expect("UDP receipt")
                .expect("Datagram"),
            ConnectionReceive::Accepted(_)
        ));
    }
    assert_eq!(callbacks, 0);
    let mut text = [0xa5; 4];
    let mut slots = [ConfigStringSlot::default()];
    let mut entities = [];
    let mut clients = [];
    let before = slots;
    let result = connection.drain(&mut [0; 64], 128, 8);
    assert_eq!(result.outcome, Ok(DrainStatus::Ready));
    let error = ReliableGameStatePlan::parse(
        connection.frame().expect("Complete malformed body").body(),
        BaselineProtocol::Cod4x21,
        128,
        BUDGET,
        &mut [0; 4],
        GameStateStorage {
            configstring_bytes: &mut text,
            configstring_slots: &mut slots,
            entity_slots: &mut entities,
            client_slots: &mut clients,
        },
    )
    .expect_err("Late extra byte blocks all publication");
    assert_eq!(error, ReliableGameStateError::TrailingBytes);
    assert_eq!(text, [0xa5; 4]);
    assert_eq!(slots, before);
    assert_eq!(connection.stats().reliable.released_frames, 0);
    assert!(connection.frame().is_some());
    assert!(connection.consume_frame()); // Explicit host discard, with no engine application.
    assert_eq!(
        connection.drain(&mut [0; 64], 128, 8).outcome,
        Ok(DrainStatus::Ready)
    );
    let plan = ReliableGameStatePlan::parse(
        connection.frame().expect("Next complete body").body(),
        BaselineProtocol::Cod4x21,
        128,
        BUDGET,
        &mut [0; 4],
        GameStateStorage {
            configstring_bytes: &mut text,
            configstring_slots: &mut slots,
            entity_slots: &mut entities,
            client_slots: &mut clients,
        },
    )
    .expect("Complete typed admission");
    assert!(connection.consume_frame()); // The cooked plan retains no network-frame borrow.
    assert_eq!(plan.game_state().client_index(), 3);
    assert_eq!(plan.game_state().config_data_sequence(), 9);
    assert_eq!(
        plan.game_state()
            .configstrings()
            .next()
            .expect("Patch")
            .bytes(),
        b"new."
    );
    assert_eq!(text, *b"new.");
    let stats = connection.stats();
    assert_eq!(stats.reliable.accepted_datagrams, 2);
    assert_eq!(stats.reliable.released_frames, 2);
    assert_eq!(stats.reliable.framing_failures, 0);
    assert_eq!(send_when_due(&mut connection, deadline).fragment, None);
    responder.join().expect("Peer");
}
