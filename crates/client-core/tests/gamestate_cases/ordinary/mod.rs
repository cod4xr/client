// SPDX-License-Identifier: AGPL-3.0-or-later
mod bounds;
mod framing;
mod vectors;

use super::{BUDGET, Output, encoded, finish, mixed, string};
use crate::service_support::Fixture;
use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    gamestate::{
        GameStateBudget,
        services::{GameStateServiceError as Error, GameStateServicePlan, GameStateServiceStorage},
    },
    service_controls::{ClientDataUpdateSlot, ClientServiceBudget, ClientServiceStorage},
};

const SERVICES: ClientServiceBudget = ClientServiceBudget {
    max_records: 8,
    max_client_updates: 2,
};

fn command(writer: &mut MessageWriter<'_>, sequence: i32, value: &[u8]) {
    writer.write_u8(4).expect("Outer command");
    writer.write_i32(sequence).expect("Command sequence");
    string(writer, value);
}
fn identity(writer: &mut MessageWriter<'_>, sequence: i32, value: &[u8]) {
    writer.write_u8(11).expect("Sequenced outer identity");
    writer.write_i32(sequence).expect("Update sequence");
    writer
        .write_u8(2)
        .expect("Outer client differs from inner client");
    string(writer, value);
    string(writer, b"Tag%");
}
fn carrier(tail: u8) -> Vec<u8> {
    encoded(|writer| {
        writer.write_u8(0).expect("Prefix NOP");
        command(writer, 4, b"before%");
        identity(writer, 1, b"Before%");
        writer.write_u8(1).expect("Outer gamestate");
        mixed(writer, 63);
        identity(writer, 2, b"After");
        command(writer, 9, b"after");
        writer.write_u8(0).expect("Tail NOP");
        writer.write_u8(tail).expect("Outer EOF or invalid tail");
    })
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Finals {
    game_state: Output,
    commands: Vec<u8>,
    identities: Vec<ClientDataUpdateSlot>,
}
impl Finals {
    fn new() -> Self {
        Self {
            game_state: Output::dirty(9, 4, 3, 3),
            commands: vec![0xa5; 14],
            identities: vec![ClientDataUpdateSlot::default(); 3],
        }
    }
    fn storage<'scratch>(
        &mut self,
        commands: &'scratch mut [u8],
        strings: &'scratch mut [u8],
        clients: &'scratch mut [ClientDataUpdateSlot],
    ) -> GameStateServiceStorage<'scratch, '_> {
        GameStateServiceStorage {
            command_scratch: commands,
            configstring_scratch: strings,
            controls: ClientServiceStorage {
                command_bytes: &mut self.commands,
                client_scratch: clients,
                client_updates: &mut self.identities,
            },
            game_state: self.game_state.storage(),
        }
    }
}
fn parse<'output>(
    source: &[u8],
    profile: BaselineProtocol,
    services: ClientServiceBudget,
    budget: GameStateBudget,
    finals: &'output mut Finals,
) -> Result<GameStateServicePlan<'output>, Error> {
    parse_with_scratch(source, profile, services, budget, finals, (16, 8, 2))
}
fn parse_with_scratch<'output>(
    source: &[u8],
    profile: BaselineProtocol,
    services: ClientServiceBudget,
    budget: GameStateBudget,
    finals: &'output mut Finals,
    scratch: (usize, usize, usize),
) -> Result<GameStateServicePlan<'output>, Error> {
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut outgoing = LegacyClientCommands::new(&mut raw).expect("Original outgoing owner");
    outgoing.enqueue(b"PrivateKey").expect("Key");
    outgoing.mark_transmitted(1).expect("Host send evidence");
    let mut fixture = Fixture::new(source, 1, b"PrivateKey");
    let payload = fixture.decode_profile(&outgoing, profile);
    let result = GameStateServicePlan::parse(
        &payload,
        services,
        budget,
        finals.storage(
            &mut vec![0; scratch.0],
            &mut vec![0; scratch.1],
            &mut vec![ClientDataUpdateSlot::default(); scratch.2],
        ),
    );
    assert_eq!(outgoing.acknowledged_sequence(), 0);
    assert_eq!(outgoing.pending_count(), 1);
    result
}
