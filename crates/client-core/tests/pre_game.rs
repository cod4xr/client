// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native pre-game token vectors and exclusive history/typed-notice transactions.
#![allow(clippy::expect_used, reason = "Finite checked native command vectors.")]
#[path = "pre_game_cases/lexical.rs"]
mod lexical;
#[path = "pre_game_cases/ownership.rs"]
mod ownership;
mod service_support;
#[path = "pre_game_cases/transactions.rs"]
mod transactions;
#[path = "pre_game_cases/vectors.rs"]
mod vectors;

use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    pre_game::{PreGameAction, PreGameBudget, PreGameError},
    server_commands::{SERVER_COMMAND_SLOT_BYTES, ServerCommandHistory, ServerCommandSlot},
    service_controls::{LegacyServicePlan, ServiceControlBudget},
};
use service_support::{Fixture, append_command};

const BUDGET: PreGameBudget = PreGameBudget {
    max_command_bytes: 16_384,
    max_events: 128,
};
fn plan<'output>(
    commands: &[(i32, &[u8])],
    message: u32,
    profile: BaselineProtocol,
    output: &'output mut [u8],
) -> LegacyServicePlan<'output> {
    let mut source = Vec::new();
    for &(sequence, bytes) in commands {
        append_command(&mut source, sequence, bytes);
    }
    source.push(7);
    let mut fixture = if message == 7 {
        Fixture::new(&source, 0, b"")
    } else {
        Fixture::at_sequence(&source, 0, b"", message)
    };
    let mut outgoing_bytes = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let outgoing = LegacyClientCommands::new(&mut outgoing_bytes).expect("Outgoing history");
    let payload = if profile == BaselineProtocol::Cod4x21 {
        fixture.decode(&outgoing)
    } else {
        fixture.decode_profile(&outgoing, profile)
    };
    LegacyServicePlan::parse(
        &payload,
        ServiceControlBudget::default(),
        &mut [0; 8191],
        output,
    )
    .expect("Whole supported service stream")
}
#[derive(Debug, Eq, PartialEq)]
enum Notice {
    Disconnect(Option<Vec<u8>>, bool),
    Map(Vec<u8>, Vec<u8>),
    Restart,
}
fn owned(action: PreGameAction<'_>) -> Notice {
    match action {
        PreGameAction::Disconnect { reason, punkbuster } => {
            Notice::Disconnect(reason.map(|text| text.as_bytes().to_vec()), punkbuster)
        }
        PreGameAction::LoadMap { map, gametype } => {
            Notice::Map(map.as_bytes().to_vec(), gametype.as_bytes().to_vec())
        }
        PreGameAction::FastRestart => Notice::Restart,
    }
}
fn inspect(bytes: &[u8]) -> Option<Notice> {
    let mut output = vec![0; 8191];
    let plan = plan(&[(1, bytes)], 7, BaselineProtocol::Cod4x21, &mut output);
    let mut retained = vec![0; SERVER_COMMAND_SLOT_BYTES];
    let mut slots = [ServerCommandSlot::default()];
    let mut history =
        ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut retained, &mut slots)
            .expect("History");
    let committed = history
        .prepare(&plan)
        .expect("Command preflight")
        .with_pre_game(BUDGET)
        .expect("Typed notice preflight")
        .commit();
    committed.events().next().map(|event| owned(event.action()))
}
