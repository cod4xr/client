// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::wire;
use super::{Buffers, gamestate::framed, send_when_due};
use crate::{
    channel_support::accept,
    support::{receive, server},
};
use cod4xr_client_core::{
    BaselineProtocol,
    client_identities::{
        ClientIdentityBudget, ClientIdentityError, ClientIdentitySlot, ClientIdentityState,
    },
    gamestate::{ClientDataSlot, GameStateBudget, GameStateStorage, ReliableGameStateError},
};
use cod4xr_client_net::{ConnectionReceive, GameStateFrameError, ReliableConnection};
use cod4xr_reliable::endpoint::{DrainStatus, EndpointConfig};
use std::{
    thread,
    time::{Duration, Instant},
};

const BODY: &[u8] = &[
    1, 0, 0, 0, 1, 7, 0, 0, 0, // Complete reliable command/header.
    11, 3, b'P', b'r', b'i', b'v', b'a', b't', b'e', b'%', 0, b'T', b'a', b'g', 0, 7, 9, 0, 0, 0,
    3, 0, 0, 0, 0x44, 0x33, 0x22, 0x11, 0, 0, 0, 0,
];
const GAME: GameStateBudget = GameStateBudget {
    max_services: 2,
    max_configstrings: 0,
    max_text_bytes: 0,
    max_entities: 0,
    max_client_records: 1,
};
const IDENTITIES: ClientIdentityBudget = ClientIdentityBudget {
    max_clients: 1,
    max_records: 1,
};
fn storage(clients: &mut [ClientDataSlot]) -> GameStateStorage<'_> {
    GameStateStorage {
        configstring_bytes: &mut [],
        configstring_slots: &mut [],
        entity_slots: &mut [],
        client_slots: clients,
    }
}
fn receive_frames(connection: &mut ReliableConnection<'_>, deadline: Instant) {
    for _ in 0..2 {
        assert!(matches!(
            connection
                .receive_until(deadline, |_| ())
                .expect("Receive")
                .expect("Datagram"),
            ConnectionReceive::Accepted(_)
        ));
    }
    assert_eq!(
        connection.drain(&mut [0; 64], 128, 8).outcome,
        Ok(DrainStatus::Ready)
    );
}

#[test]
fn udp_frame_identity_baseline_cancels_or_commits_both_owners_after_complete_preflight() {
    let socket = server();
    let address = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        let mut bad = BODY.to_vec();
        bad.push(0);
        socket
            .send_to(&wire(1, 0, &[], 4, &framed(BODY)), client)
            .expect("Later complete frame");
        socket
            .send_to(&wire(0, 0, &[], 4, &framed(&bad)), client)
            .expect("Earlier malformed frame");
        let (bytes, source) = receive(&socket);
        assert_eq!(source, client);
        assert_eq!(bytes, wire(-1, 2, &[], 4, b""));
    });
    let mut buffers = Buffers::new();
    buffers.body.resize(128, 0xa5);
    let mut connection = buffers
        .connect(address, 21, EndpointConfig::default())
        .expect("Original owner");
    let deadline = Instant::now() + Duration::from_secs(4);
    receive_frames(&mut connection, deadline);
    let mut cells = vec![ClientIdentitySlot::default(); 1];
    let mut identities =
        ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut cells).expect("Identity owner");
    let mut clients = vec![ClientDataSlot::default(); 1];
    let before = clients.clone();
    let mut scratch = vec![ClientIdentitySlot::default(); 1];
    assert!(matches!(
        connection.prepare_game_state(128, GAME, &mut [], storage(&mut clients)),
        Err(GameStateFrameError::Decode(
            ReliableGameStateError::TrailingBytes
        ))
    ));
    assert_eq!(clients, before);
    assert_eq!(identities.accepted_sequence(), None);
    assert!(connection.consume_frame());
    assert_eq!(
        connection.drain(&mut [0; 64], 128, 8).outcome,
        Ok(DrainStatus::Ready)
    );
    let received_at = connection.stats().reliable.last_accepted_at;
    let frame = connection
        .prepare_game_state(128, GAME, &mut [], storage(&mut clients))
        .expect("Whole frame");
    assert_eq!(
        frame
            .with_client_identities(&mut identities, IDENTITIES, &mut [])
            .err(),
        Some(ClientIdentityError::ScratchCapacity)
    );
    assert!(connection.frame().is_some());
    assert_eq!(identities.generation(), 0);
    let joint = connection
        .prepare_game_state(128, GAME, &mut [], storage(&mut clients))
        .expect("Same frame")
        .with_client_identities(&mut identities, IDENTITIES, &mut scratch)
        .expect("Joint preflight");
    assert_eq!(joint.identity_change().sequence, 9);
    assert_eq!(joint.clients().next().expect("Staged").name(), b"Private.");
    assert_eq!(joint.plan().game_state().client_index(), 3);
    drop(joint);
    assert!(connection.frame().is_some());
    assert_eq!(identities.accepted_sequence(), None);
    assert_eq!(identities.len(), 0);
    let joint = connection
        .prepare_game_state(128, GAME, &mut [], storage(&mut clients))
        .expect("Retry")
        .with_client_identities(&mut identities, IDENTITIES, &mut scratch)
        .expect("Joint retry");
    assert!(!format!("{joint:?}").contains("Private"));
    let (plan, change) = joint.commit();
    assert_eq!(change.clients, 1);
    assert!(connection.frame().is_none());
    assert_eq!(identities.generation(), 1);
    assert_eq!(identities.accepted_sequence(), Some(9));
    assert_eq!(connection.stats().reliable.committed_gamestates, 1);
    assert_eq!(connection.stats().reliable.last_accepted_at, received_at);
    assert_eq!(send_when_due(&mut connection, deadline).fragment, None);
    responder.join().expect("Peer");
    drop(connection);
    buffers.body.fill(0);
    assert_eq!(plan.game_state().client_index(), 3);
    clients.fill(ClientDataSlot::default());
    scratch.fill(ClientIdentitySlot::default());
    assert_eq!(
        identities
            .client(3)
            .expect("Retained owned identity")
            .name(),
        b"Private."
    );
    assert_eq!(identities.client(3).expect("Tag").clan_tag(), b"Tag");
    assert_eq!(identities.own_client_index(), Some(3));
}
