// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    GameStateServiceError as Error, GameStateServicePlan, GameStateServicePrefix,
    GameStateServiceStorage,
};
use crate::{
    BaselineProtocol,
    gamestate::{GameStateBudget, GameStatePlan},
    message_body::BodyReader,
    server_payload::DecodedServerPayload,
    service_controls::{
        ClientServiceBudget, ClientServiceStorage, LegacyServicePlan, ServiceControlBudget,
        ServiceControlError, client_scan::Clients, publish, scan::Layout,
    },
};
use cod4x_protocol::MAX_MESSAGE_BYTES;

pub(super) fn parse<'output>(
    payload: &DecodedServerPayload<'_, '_>,
    services: ClientServiceBudget,
    budget: GameStateBudget,
    storage: GameStateServiceStorage<'_, 'output>,
) -> Result<GameStateServicePlan<'output>, Error> {
    if payload.profile() != BaselineProtocol::Cod4x21 {
        return Err(Error::Profile);
    }
    let GameStateServiceStorage {
        command_scratch,
        configstring_scratch,
        controls,
        game_state: storage,
    } = storage;
    let ClientServiceStorage {
        command_bytes,
        client_scratch,
        client_updates,
    } = controls;
    let limits = ServiceControlBudget {
        max_records: services.max_records,
    };
    let mut layout = Layout::new(limits).map_err(Error::Controls)?;
    let body = payload.body();
    let mut reader = BodyReader::try_new(body, MAX_MESSAGE_BYTES)
        .map_err(|error| Error::Controls(ServiceControlError::from(error)))?;
    let mut clients = Clients::Accept {
        maximum: services.max_client_updates,
        capacity: client_updates.len(),
        scratch: &mut *client_scratch,
    };
    let prefix = inspect_prefix(
        &mut reader,
        &mut layout,
        limits,
        command_scratch,
        command_bytes.len(),
        &mut clients,
    )?;
    let start = reader.clone();
    let game_layout = GameStatePlan::inspect(
        &mut reader,
        payload.profile(),
        budget,
        configstring_scratch,
        &storage,
    )
    .map_err(Error::GameState)?;
    loop {
        let opcode = layout.next(&mut reader, limits).map_err(Error::Controls)?;
        if layout
            .control(
                opcode,
                &mut reader,
                command_scratch,
                command_bytes.len(),
                &mut clients,
            )
            .map_err(Error::Controls)?
        {
            break;
        }
    }
    // Complete immutable inner/trailer/outer/EOF validation precedes all writes.
    // Closed replay uses the same capacities, source spans and private scratch.
    let game_state = GameStatePlan::publish(
        &start,
        payload.profile(),
        budget,
        configstring_scratch,
        storage,
        game_layout,
    )
    .map_err(Error::GameState)?;
    publish::commands(body, &layout, command_bytes).map_err(Error::Controls)?;
    let count = layout.client_update_count;
    client_updates[..count].clone_from_slice(&client_scratch[..count]);
    let controls = LegacyServicePlan::from_parts(
        payload.profile(),
        payload.sequence(),
        payload.client_command_acknowledge(),
        layout,
        command_bytes,
        &client_updates[..count],
    );
    Ok(GameStateServicePlan {
        controls,
        game_state,
        prefix,
    })
}

fn inspect_prefix(
    reader: &mut BodyReader<'_>,
    layout: &mut Layout,
    limits: ServiceControlBudget,
    scratch: &mut [u8],
    command_capacity: usize,
    clients: &mut Clients<'_>,
) -> Result<GameStateServicePrefix, Error> {
    loop {
        let records = layout.record_count;
        let opcode = layout.next(reader, limits).map_err(Error::Controls)?;
        if opcode == 1 {
            return Ok(GameStateServicePrefix {
                records,
                commands: layout.command_count,
                client_updates: layout.client_update_count,
            });
        }
        if layout
            .control(opcode, reader, scratch, command_capacity, clients)
            .map_err(Error::Controls)?
        {
            return Err(Error::MissingGameState);
        }
    }
}
