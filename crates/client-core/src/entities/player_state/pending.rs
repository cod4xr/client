// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{
    EntityDecodeContext,
    hud::SnapshotHud,
    objectives::SnapshotObjectives,
    player_scalars::{PlayerScalarPrefix, PlayerScalars, word},
};
use super::{
    LivePlayerState, PlayerStateError, PredictionProvenance, PredictionRequest, PredictionSample,
    PredictionWords, state::DecodedPlayerState,
};
use crate::{
    BaselineProtocol,
    message_body::BodyReader,
    player_arrays::{PlayerAmmunition, PlayerStats, WeaponModels},
};

// Owned unresolved words for enclosing transactions, never a public admitted state.
pub(crate) struct PendingPlayerState {
    decoded: DecodedPlayerState,
    request: Option<PredictionRequest>,
    fallback: PredictionWords,
}
impl PendingPlayerState {
    pub(crate) fn decode_candidate(
        reader: &mut BodyReader<'_>,
        context: EntityDecodeContext,
        time: u32,
        reference: Option<&LivePlayerState>,
    ) -> Result<Self, PlayerStateError> {
        let profile = context.profile();
        if reference.is_some_and(|prior| prior.profile() != profile) {
            return Err(PlayerStateError::ReferenceProfile);
        }
        let prefix = PlayerScalarPrefix::parse_live(
            reader,
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
        let stats = PlayerStats::parse(reader, profile, reference.map(LivePlayerState::stats))
            .map_err(PlayerStateError::Array)?;
        let ammunition =
            PlayerAmmunition::parse(reader, profile, reference.map(LivePlayerState::ammunition))
                .map_err(PlayerStateError::Array)?;
        let objectives =
            SnapshotObjectives::parse(reader, profile, reference.map(LivePlayerState::objectives))
                .map_err(PlayerStateError::Objective)?;
        let hud = SnapshotHud::parse(reader, context, time, reference.map(LivePlayerState::hud))
            .map_err(PlayerStateError::Hud)?;
        let models = WeaponModels::parse(reader, profile, reference.map(LivePlayerState::models))
            .map_err(PlayerStateError::Array)?;
        let fallback = reference.map_or_else(
            || PredictionWords::from_scalars(&PlayerScalars::zero(profile)),
            |prior| PredictionWords::from_scalars(prior.scalars()),
        );
        Ok(Self {
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
    pub(crate) fn profile(&self) -> BaselineProtocol {
        self.decoded.context.profile()
    }
    pub(crate) fn request(&self) -> Option<PredictionRequest> {
        self.request
    }
    pub(crate) fn transmitted(self) -> Result<LivePlayerState, PlayerStateError> {
        if self.request.is_some() {
            return Err(PlayerStateError::PredictionRequired);
        }
        Ok(self.finish(PredictionProvenance::Transmitted))
    }
    pub(crate) fn predicted(
        mut self,
        sample: &PredictionSample,
    ) -> Result<LivePlayerState, PlayerStateError> {
        let request = self.request.ok_or(PlayerStateError::UnexpectedPrediction)?;
        if sample.profile() != request.profile() {
            return Err(PlayerStateError::PredictionProfile);
        }
        if sample.command_time() != request.command_time() {
            return Err(PlayerStateError::PredictionTime);
        }
        sample.words.apply(&mut self.decoded.scalars);
        Ok(self.finish(PredictionProvenance::CommandTimeMatched))
    }
    pub(crate) fn prior_fallback(mut self) -> Result<LivePlayerState, PlayerStateError> {
        if self.request.is_none() {
            return Err(PlayerStateError::UnexpectedPrediction);
        }
        self.fallback.apply(&mut self.decoded.scalars);
        Ok(self.finish(PredictionProvenance::ExplicitPriorFallback))
    }
    fn finish(self, prediction: PredictionProvenance) -> LivePlayerState {
        LivePlayerState {
            decoded: self.decoded,
            prediction,
        }
    }
}
