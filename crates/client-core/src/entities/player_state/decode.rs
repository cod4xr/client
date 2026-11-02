// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{
    EntityDecodeContext,
    hud::SnapshotHud,
    objectives::SnapshotObjectives,
    player_scalars::{PlayerScalarPrefix, PlayerScalars, word},
};
use super::{
    LivePlayerState, PlayerStateError, PredictionRequest, PredictionWords, PreparedPlayerState,
    state::DecodedPlayerState,
};
use crate::{
    message_body::BodyReader,
    player_arrays::{PlayerAmmunition, PlayerStats, WeaponModels},
};

pub(super) fn prepare<'reader, 'body>(
    reader: &'reader mut BodyReader<'body>,
    context: EntityDecodeContext,
    time: u32,
    reference: Option<&LivePlayerState>,
) -> Result<PreparedPlayerState<'reader, 'body>, PlayerStateError> {
    let profile = context.profile();
    if reference.is_some_and(|prior| prior.profile() != profile) {
        return Err(PlayerStateError::ReferenceProfile);
    }
    let initial_position = reader.byte_position();
    let mut candidate = reader.clone();
    let prefix = PlayerScalarPrefix::parse_live(
        &mut candidate,
        profile,
        time,
        reference.map(LivePlayerState::scalars),
    )
    .map_err(PlayerStateError::Scalar)?;
    let request = prefix.needs_prediction().then(|| PredictionRequest {
        profile,
        command_time: prefix.scalars().word(word::session::COMMAND_TIME),
    });
    let scalars = prefix.into_scalars();
    let stats = PlayerStats::parse(
        &mut candidate,
        profile,
        reference.map(LivePlayerState::stats),
    )
    .map_err(PlayerStateError::Array)?;
    let ammunition = PlayerAmmunition::parse(
        &mut candidate,
        profile,
        reference.map(LivePlayerState::ammunition),
    )
    .map_err(PlayerStateError::Array)?;
    let objectives = SnapshotObjectives::parse(
        &mut candidate,
        profile,
        reference.map(LivePlayerState::objectives),
    )
    .map_err(PlayerStateError::Objective)?;
    let hud = SnapshotHud::parse(
        &mut candidate,
        context,
        time,
        reference.map(LivePlayerState::hud),
    )
    .map_err(PlayerStateError::Hud)?;
    let models = WeaponModels::parse(
        &mut candidate,
        profile,
        reference.map(LivePlayerState::models),
    )
    .map_err(PlayerStateError::Array)?;
    let fallback = reference.map_or_else(
        || PredictionWords::from_scalars(&PlayerScalars::zero(profile)),
        |prior| PredictionWords::from_scalars(prior.scalars()),
    );
    Ok(PreparedPlayerState {
        consumed_bytes: candidate.byte_position() - initial_position,
        reader,
        candidate,
        request,
        fallback,
        decoded: DecodedPlayerState {
            context,
            scalars,
            stats,
            ammunition,
            objectives,
            hud,
            models,
        },
    })
}
