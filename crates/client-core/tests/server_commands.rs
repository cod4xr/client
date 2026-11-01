// SPDX-License-Identifier: AGPL-3.0-or-later
//! Configurable retention, tagged reuse and prepared incoming/outgoing history updates.
#![allow(
    clippy::expect_used,
    reason = "Finite explicit metadata/data and validated plan fixtures."
)]
mod service_support;
#[path = "server_command_cases/storage.rs"]
mod storage;
#[path = "server_command_cases/updates.rs"]
mod updates;

use cod4xr_client_core::{
    commands::{ClientCommandError, LegacyClientCommands},
    server_commands::{ServerCommandHistory, ServerCommandUpdate},
    service_controls::{LegacyServicePlan, ServiceControlBudget},
};
use service_support::{Fixture, append_command};

fn plan<'output>(
    fixture: &mut Fixture,
    client: &LegacyClientCommands<'_>,
    output: &'output mut [u8],
) -> LegacyServicePlan<'output> {
    let payload = fixture.decode(client);
    LegacyServicePlan::parse(
        &payload,
        ServiceControlBudget::default(),
        &mut [0; 8191],
        output,
    )
    .expect("Validated services")
}

fn apply_pair(
    server: &mut ServerCommandHistory<'_>,
    client: &mut LegacyClientCommands<'_>,
    plan: &LegacyServicePlan<'_>,
) -> Result<ServerCommandUpdate, ClientCommandError> {
    let prepared = server
        .prepare(plan)
        .expect("Incoming resources preflighted");
    client.commit_acknowledgement(plan.client_command_acknowledge())?;
    Ok(prepared.commit())
}
