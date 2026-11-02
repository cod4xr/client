// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::SnapshotError;
use crate::{service_controls::ServiceControlError, snapshots::services::SnapshotPayloadError};
use std::{error::Error, fmt};

/// Bounded mixed-carrier failures without command text, names or snapshot words.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MixedSnapshotError {
    /// Caller context disagrees with the actual decoded payload.
    Binding(SnapshotPayloadError),
    /// A command, identity, opcode or service/retention bound failed.
    Controls(ServiceControlError),
    /// Snapshot binding/decoding or explicit prediction selection failed.
    Snapshot(SnapshotError),
    /// Logical EOF arrived without a snapshot in this specialized carrier.
    MissingSnapshot,
}
impl fmt::Display for MixedSnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Binding(_) => "mixed snapshot decoded-payload context mismatch",
            Self::Controls(_) => "invalid mixed snapshot service controls",
            Self::Snapshot(_) => "invalid mixed snapshot state or prediction",
            Self::MissingSnapshot => "mixed snapshot carrier requires a snapshot",
        })
    }
}
impl Error for MixedSnapshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Binding(error) => Some(error),
            Self::Controls(error) => Some(error),
            Self::Snapshot(error) => Some(error),
            Self::MissingSnapshot => None,
        }
    }
}
