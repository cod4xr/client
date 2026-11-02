// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    PreparedSnapshot, SnapshotDelta, SnapshotError, SnapshotHeader, SnapshotInput, SnapshotOutput,
    transaction::Transaction,
};
use crate::{
    message_body::BodyReader,
    player_state::PendingPlayerState,
    snapshot_clients::{ClientStateSource, sink as client_sink, walk as client_walk},
    snapshot_entities::{
        EntityStateSource, EntityStreamSources, sink as entity_sink, walk as entity_walk,
    },
};

pub(super) fn prepare<'reader, 'body, 'source, 'storage>(
    reader: &'reader mut BodyReader<'body>,
    input: SnapshotInput<'source>,
    output: SnapshotOutput<'storage>,
) -> Result<PreparedSnapshot<'reader, 'body, 'source, 'storage>, SnapshotError> {
    let candidate = reader.clone();
    prepare_at(reader, candidate, input, output)
}

pub(super) fn prepare_at<'reader, 'body, 'source, 'storage>(
    reader: &'reader mut BodyReader<'body>,
    mut candidate: BodyReader<'body>,
    input: SnapshotInput<'source>,
    output: SnapshotOutput<'storage>,
) -> Result<PreparedSnapshot<'reader, 'body, 'source, 'storage>, SnapshotError> {
    check_bindings(input)?;
    let profile = input.context.profile();
    let entity_sources = EntityStreamSources {
        previous: input
            .previous
            .map_or(EntityStateSource::from_states(&[]), |prior| prior.entities),
        baselines: input.baselines.entities,
    };
    let client_source = input
        .previous
        .map_or(ClientStateSource::from_states(&[]), |prior| prior.clients);
    entity_sources
        .previous
        .validate(input.budget.entities.max_previous_entities, false)
        .map_err(SnapshotError::Entities)?;
    entity_sources
        .baselines
        .validate(input.budget.entities.max_baselines, true)
        .map_err(SnapshotError::Entities)?;
    client_source
        .validate(input.budget.clients.max_previous_clients)
        .map_err(SnapshotError::Clients)?;
    let header = SnapshotHeader::parse(&mut candidate, profile).map_err(SnapshotError::Body)?;
    check_reference(header, input)?;
    let player = PendingPlayerState::decode_candidate(
        &mut candidate,
        input.context,
        header.server_time(),
        input.previous.map(|prior| prior.player),
    )
    .map_err(SnapshotError::Player)?;
    let stream_start = candidate.clone();
    let entity_input = entity_walk::Input {
        context: input.context,
        time: header.server_time(),
        sources: entity_sources,
        budget: input.budget.entities,
        capacity: output.entities.len(),
    };
    let entity_stats = entity_walk::walk(&mut candidate, entity_input, &mut entity_sink::Noop)
        .map_err(SnapshotError::Entities)?;
    let entity_bytes = candidate.byte_position() - stream_start.byte_position();
    let client_start = candidate.byte_position();
    let client_input = client_walk::Input {
        profile,
        previous: client_source,
        budget: input.budget.clients,
        capacity: output.clients.len(),
    };
    let client_stats = client_walk::walk(&mut candidate, client_input, &mut client_sink::Noop)
        .map_err(SnapshotError::Clients)?;
    let client_bytes = candidate.byte_position() - client_start;
    Ok(PreparedSnapshot {
        player,
        transaction: Transaction {
            consumed_bytes: candidate.byte_position() - reader.byte_position(),
            reader,
            candidate,
            stream_start,
            header,
            message: input.message,
            output,
            entity_input,
            client_input,
            entity_stats,
            client_stats,
            entity_bytes,
            client_bytes,
        },
    })
}
fn check_bindings(input: SnapshotInput<'_>) -> Result<(), SnapshotError> {
    if input.baselines.epoch != input.message.epoch() {
        return Err(SnapshotError::BaselineEpoch);
    }
    if input.baselines.profile != input.context.profile() {
        return Err(SnapshotError::BaselineProfile);
    }
    if let Some(prior) = input.previous {
        if prior.message.epoch() != input.message.epoch() {
            return Err(SnapshotError::ReferenceEpoch);
        }
        if prior.profile != input.context.profile() {
            return Err(SnapshotError::ReferenceProfile);
        }
    }
    Ok(())
}
fn check_reference(header: SnapshotHeader, input: SnapshotInput<'_>) -> Result<(), SnapshotError> {
    match (header.delta(), input.previous) {
        (SnapshotDelta::Full, None) => Ok(()),
        (SnapshotDelta::Full, Some(_)) => Err(SnapshotError::UnexpectedReference),
        (SnapshotDelta::Previous(_), None) => Err(SnapshotError::MissingReference),
        (SnapshotDelta::Previous(distance), Some(prior)) => {
            if input
                .message
                .sequence()
                .checked_sub(u32::from(distance.get()))
                != Some(prior.message.sequence())
            {
                return Err(SnapshotError::ReferenceSequence);
            }
            Ok(())
        }
    }
}
