// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::user_commands::UserCommandEncodeError;
use std::{error::Error, fmt};

/// Fixed movement batch failures without keys, prediction data or input values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MovementMessageError {
    /// Native batch requires one through thirty-two commands.
    CommandCount,
    /// Prediction report belongs to another explicit profile.
    PredictionProfile,
    /// Supplied snapshot belongs to another explicit profile.
    SnapshotProfile,
    /// A supplied snapshot is newer than the admitted message acknowledgement.
    SnapshotSequence,
    /// A native command's fields or conditional action presence are invalid.
    Command(UserCommandEncodeError),
}
impl fmt::Display for MovementMessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::CommandCount => "movement batch exceeds native command-count bounds",
            Self::PredictionProfile => "movement prediction profile mismatch",
            Self::SnapshotProfile => "movement snapshot profile mismatch",
            Self::SnapshotSequence => "movement snapshot is newer than the message acknowledgement",
            Self::Command(_) => "invalid native movement command",
        })
    }
}
impl Error for MovementMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Command(error) => Some(error),
            _ => None,
        }
    }
}
