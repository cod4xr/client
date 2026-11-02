// SPDX-License-Identifier: AGPL-3.0-or-later
//! Atomic identity application with independent native carrier bytes and map models.
#![allow(
    clippy::expect_used,
    reason = "Finite checked identity-state fixtures."
)]
#[path = "identity_cases/baselines.rs"]
mod baselines;
#[path = "identity_cases/model.rs"]
mod model;
#[path = "identity_cases/transactions.rs"]
mod transactions;
#[path = "identity_cases/updates.rs"]
mod updates;
use cod4xr_client_core::{
    BaselineProtocol,
    client_data::{ClientData, ClientDataUpdate},
    client_identities::{
        ClientIdentityBudget, ClientIdentityError, ClientIdentitySlot, ClientIdentityState,
    },
    gamestate::{ClientDataSlot, GameStateBudget, GameStateStorage, ReliableGameStatePlan},
    message_body::BodyReader,
};

const BUDGET: ClientIdentityBudget = ClientIdentityBudget {
    max_clients: 64,
    max_records: 1024,
};
type Record<'a> = (u8, &'a [u8], &'a [u8]);
fn identity(index: u8, name: &[u8], tag: &[u8]) -> Vec<u8> {
    let mut bytes = vec![index];
    bytes.extend_from_slice(name);
    bytes.push(0);
    bytes.extend_from_slice(tag);
    bytes.push(0);
    bytes
}
fn baseline<'storage>(
    sequence: u32,
    own: u32,
    records: &[Record<'_>],
    storage: &'storage mut [ClientDataSlot],
) -> ReliableGameStatePlan<'storage> {
    let mut bytes = 1_i32.to_le_bytes().to_vec();
    bytes.push(1);
    bytes.extend_from_slice(&7_u32.to_le_bytes());
    for &(index, name, tag) in records {
        bytes.push(11);
        bytes.extend_from_slice(&identity(index, name, tag));
    }
    bytes.push(7);
    for value in [sequence, own, 0x1234_5678, 0] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    ReliableGameStatePlan::parse(
        &bytes,
        BaselineProtocol::Cod4x21,
        bytes.len(),
        GameStateBudget {
            max_services: records.len() + 1,
            max_configstrings: 0,
            max_text_bytes: 0,
            max_entities: 0,
            max_client_records: records.len(),
        },
        &mut [],
        GameStateStorage {
            configstring_bytes: &mut [],
            configstring_slots: &mut [],
            entity_slots: &mut [],
            client_slots: storage,
        },
    )
    .expect("Whole independent gamestate wrapper")
}
fn update(sequence: u32, index: u8, name: &[u8]) -> ClientDataUpdate {
    let mut bytes = sequence.to_le_bytes().to_vec();
    bytes.extend_from_slice(&identity(index, name, b"Tag%"));
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Owned update carrier");
    ClientDataUpdate::parse(&mut reader, BaselineProtocol::Cod4x21).expect("Complete update")
}
fn seed(state: &mut ClientIdentityState<'_>, sequence: u32, records: &[Record<'_>]) {
    let mut storage = vec![ClientDataSlot::default(); records.len()];
    let plan = baseline(sequence, 3, records, &mut storage);
    let mut scratch = vec![ClientIdentitySlot::default(); 64];
    let _ = state
        .prepare_game_state(&plan, BUDGET, &mut scratch)
        .expect("Baseline preflight")
        .commit();
}
#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    generation: u64,
    sequence: Option<u32>,
    own: Option<u32>,
    records: Vec<(u32, Vec<u8>, Vec<u8>)>,
}
fn snapshot(state: &ClientIdentityState<'_>) -> Snapshot {
    Snapshot {
        generation: state.generation(),
        sequence: state.accepted_sequence(),
        own: state.own_client_index(),
        records: state
            .clients()
            .map(|data| (data.index(), data.name().to_vec(), data.clan_tag().to_vec()))
            .collect(),
    }
}
