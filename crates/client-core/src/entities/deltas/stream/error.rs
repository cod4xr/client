// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::EntityDeltaError;
use crate::message_body::BodyReadError;
use std::{error::Error, fmt};

/// Fixed substream failures, excluding raw entity words, coordinates or packet data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntityStreamError {
    /// Prior snapshot source exceeds host inspection policy.
    PreviousBudget,
    /// Initial baseline source exceeds host inspection policy.
    BaselineBudget,
    /// A source is unordered, duplicated, uninitialized or outside its native namespace.
    Source {
        /// True for initial baselines; false for the previous snapshot source.
        baseline: bool,
    },
    /// Wire-index work exhausted before a complete terminator.
    RecordBudget,
    /// Merged entity count exceeds host retention policy.
    EntityBudget,
    /// Final caller cells cannot hold the complete merged result.
    OutputCapacity,
    /// Shared checked index/primitive reading failed.
    Body(BodyReadError),
    /// A complete individual entity record failed without publication.
    Delta(EntityDeltaError),
}
impl fmt::Display for EntityStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::PreviousBudget => "snapshot entity prior-source budget exhausted",
            Self::BaselineBudget => "snapshot entity baseline-source budget exhausted",
            Self::Source { .. } => "invalid ordered snapshot entity source",
            Self::RecordBudget => "snapshot entity wire-record budget exhausted",
            Self::EntityBudget => "snapshot entity output budget exhausted",
            Self::OutputCapacity => "snapshot entity final capacity insufficient",
            Self::Body(_) => "invalid or truncated snapshot entity index",
            Self::Delta(_) => "invalid or truncated snapshot entity record",
        })
    }
}
impl Error for EntityStreamError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            Self::Delta(error) => Some(error),
            _ => None,
        }
    }
}
impl From<BodyReadError> for EntityStreamError {
    fn from(error: BodyReadError) -> Self {
        Self::Body(error)
    }
}
impl From<EntityDeltaError> for EntityStreamError {
    fn from(error: EntityDeltaError) -> Self {
        Self::Delta(error)
    }
}
