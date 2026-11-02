// SPDX-License-Identifier: AGPL-3.0-or-later
//! Mixed control/identity whole-stream publication and application fixtures.
#![allow(clippy::expect_used, reason = "Finite checked mixed-service vectors.")]
#[path = "mixed_service_cases/application.rs"]
mod application;
#[path = "mixed_service_cases/bounds.rs"]
mod bounds;
use crate::service_support;
#[path = "mixed_service_cases/vectors.rs"]
mod vectors;
use cod4xr_client_core::{
    BaselineProtocol,
    client_data::ClientDataUpdate,
    client_identities::{
        ClientIdentityBudget, ClientIdentityError, ClientIdentitySlot, ClientIdentityState,
    },
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    gamestate::{GameStateBudget, GameStateStorage, ReliableGameStatePlan},
    service_controls::{
        ClientDataUpdateSlot, ClientServiceBudget, ClientServiceStorage, LegacyServicePlan,
        ServiceControlBudget, ServiceControlError,
    },
};
use service_support::{Fixture, append_command};

const BUDGET: ClientServiceBudget = ClientServiceBudget {
    max_records: 16,
    max_client_updates: 8,
};
#[derive(Clone, Debug, Eq, PartialEq)]
struct Output {
    bytes: Vec<u8>,
    updates: Vec<ClientDataUpdateSlot>,
}
impl Output {
    fn new(bytes: usize, updates: usize) -> Self {
        Self {
            bytes: vec![0xa5; bytes],
            updates: vec![ClientDataUpdateSlot::default(); updates],
        }
    }
    fn storage<'scratch, 'output>(
        &'output mut self,
        scratch: &'scratch mut [ClientDataUpdateSlot],
    ) -> ClientServiceStorage<'scratch, 'output> {
        ClientServiceStorage {
            command_bytes: &mut self.bytes,
            client_updates: &mut self.updates,
            client_scratch: scratch,
        }
    }
    fn dirty(bytes: usize, updates: usize) -> Self {
        let mut source = Vec::new();
        append_update(&mut source, 99, 3, b"OldPrivate", b"");
        source.push(7);
        let mut fixture = Fixture::new(&source, 0, b"");
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let history = LegacyClientCommands::new(&mut raw).expect("History");
        let payload = fixture.decode(&history);
        let mut output = Self::new(bytes, 1);
        let _ = LegacyServicePlan::parse_with_client_data(
            &payload,
            BUDGET,
            &mut [],
            output.storage(&mut [ClientDataUpdateSlot::default()]),
        )
        .expect("Dirty seed");
        output.updates = vec![output.updates[0].clone(); updates];
        output
    }
}
fn append_update(bytes: &mut Vec<u8>, sequence: u32, index: u8, name: &[u8], tag: &[u8]) {
    bytes.push(11);
    bytes.extend_from_slice(&sequence.to_le_bytes());
    bytes.push(index);
    bytes.extend_from_slice(name);
    bytes.push(0);
    bytes.extend_from_slice(tag);
    bytes.push(0);
}
fn mixed() -> Vec<u8> {
    let mut source = vec![0];
    append_command(&mut source, 1, b"Private%Command");
    append_update(&mut source, 10, 3, b"Player%", b"Clan%");
    source.push(0);
    append_update(&mut source, 12, 3, b"Gap", b"");
    append_update(&mut source, 11, 63, b"Other\xff", b"");
    append_command(&mut source, 2, b"");
    source.push(7);
    source
}
fn seed(state: &mut ClientIdentityState<'_>, sequence: u32) {
    let mut input = vec![1, 0, 0, 0, 1, 0, 0, 0, 0, 7];
    for value in [sequence, 3, 0x1234_5678, 0] {
        input.extend_from_slice(&value.to_le_bytes());
    }
    let plan = ReliableGameStatePlan::parse(
        &input,
        BaselineProtocol::Cod4x21,
        input.len(),
        GameStateBudget {
            max_services: 1,
            max_configstrings: 0,
            max_text_bytes: 0,
            max_entities: 0,
            max_client_records: 0,
        },
        &mut [],
        GameStateStorage {
            configstring_bytes: &mut [],
            configstring_slots: &mut [],
            entity_slots: &mut [],
            client_slots: &mut [],
        },
    )
    .expect("Whole empty baseline");
    let _ = state
        .prepare_game_state(
            &plan,
            ClientIdentityBudget {
                max_clients: 2,
                max_records: 0,
            },
            &mut [],
        )
        .expect("Seed state")
        .commit();
}
