// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{hud::HudError, objectives::ObjectiveError, player_scalars::PlayerScalarError};
use crate::player_arrays::PlayerArrayError;
use std::{error::Error, fmt};

/// Fixed-category failures without peer values or prediction command times.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerStateError {
    /// The prior belongs to another explicit baseline profile.
    ReferenceProfile,
    /// Scalar decoding failed before reader publication.
    Scalar(PlayerScalarError),
    /// Stats, ammunition or model decoding failed.
    Array(PlayerArrayError),
    /// Objective decoding failed.
    Objective(ObjectiveError),
    /// HUD decoding failed.
    Hud(HudError),
    /// Unsent origin mode requires a sample or explicit prior fallback.
    PredictionRequired,
    /// Transmitted origin mode does not accept prediction replacement.
    UnexpectedPrediction,
    /// The prediction sample belongs to another baseline profile.
    PredictionProfile,
    /// The sample does not match the decoded command time exactly.
    PredictionTime,
}
impl fmt::Display for PlayerStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ReferenceProfile => "player state reference profile mismatch",
            Self::Scalar(_) => "invalid player state scalar prefix",
            Self::Array(_) => "invalid player state array section",
            Self::Objective(_) => "invalid player state objective section",
            Self::Hud(_) => "invalid player state HUD section",
            Self::PredictionRequired => "player state requires prediction resolution",
            Self::UnexpectedPrediction => "player state does not request prediction",
            Self::PredictionProfile => "prediction sample profile mismatch",
            Self::PredictionTime => "prediction sample command time mismatch",
        })
    }
}
impl Error for PlayerStateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Scalar(error) => Some(error),
            Self::Array(error) => Some(error),
            Self::Objective(error) => Some(error),
            Self::Hud(error) => Some(error),
            _ => None,
        }
    }
}
