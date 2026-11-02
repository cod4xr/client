// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    message_body::BodyReadError,
    player_state::{PlayerStateError, history::PredictionHistoryError},
    snapshot_clients::ClientStreamError,
    snapshot_entities::EntityStreamError,
};
use std::{error::Error, fmt};

/// Metadata-only whole-snapshot constituent and binding failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotError {
    /// The containing message sequence is zero or outside the ordinary wire range.
    MessageSequence,
    /// Initial baselines belong to another caller-managed world epoch.
    BaselineEpoch,
    /// Initial baselines belong to another explicitly selected profile.
    BaselineProfile,
    /// The prior snapshot belongs to another caller-managed world epoch.
    ReferenceEpoch,
    /// The prior snapshot belongs to another explicitly selected profile.
    ReferenceProfile,
    /// The prior sequence does not exactly match the wire's message distance.
    ReferenceSequence,
    /// A nonzero wire distance requires an exact cooked prior snapshot.
    MissingReference,
    /// A full wire snapshot does not accept a previous snapshot reference.
    UnexpectedReference,
    /// Header reading failed.
    Body(BodyReadError),
    /// Player-state decoding or prediction resolution failed.
    Player(PlayerStateError),
    /// Exact retained prediction lookup or world/profile binding failed.
    PredictionHistory(PredictionHistoryError),
    /// Entity stream validation failed.
    Entities(EntityStreamError),
    /// Client-state stream validation failed.
    Clients(ClientStreamError),
}
impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MessageSequence => "invalid snapshot message sequence",
            Self::BaselineEpoch => "snapshot baseline epoch mismatch",
            Self::BaselineProfile => "snapshot baseline profile mismatch",
            Self::ReferenceEpoch => "snapshot reference epoch mismatch",
            Self::ReferenceProfile => "snapshot reference profile mismatch",
            Self::ReferenceSequence => "snapshot reference sequence mismatch",
            Self::MissingReference => "snapshot requires an exact prior reference",
            Self::UnexpectedReference => "full snapshot cannot use a prior reference",
            Self::Body(_) => "invalid snapshot header",
            Self::Player(_) => "invalid snapshot player state or prediction",
            Self::PredictionHistory(_) => "snapshot prediction history rejected",
            Self::Entities(_) => "invalid snapshot entity stream",
            Self::Clients(_) => "invalid snapshot client-state stream",
        })
    }
}
impl Error for SnapshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            Self::Player(error) => Some(error),
            Self::PredictionHistory(error) => Some(error),
            Self::Entities(error) => Some(error),
            Self::Clients(error) => Some(error),
            _ => None,
        }
    }
}
