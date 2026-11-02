// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::ServerPayloadError;
use crate::{commands::ClientCommandError, snapshots::mixed_services::MixedSnapshotError};
use std::{error::Error, fmt};

/// Metadata-only mixed-carrier original-owner ACK preparation failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MixedSnapshotAcknowledgementError {
    /// Shared header, compatibility transform or decompression failed.
    Decode(ServerPayloadError),
    /// The original retained/transmitted client-command prefix was rejected.
    Acknowledgement(ClientCommandError),
    /// Complete mixed controls/snapshot/EOF or prediction validation failed.
    Payload(MixedSnapshotError),
}
impl fmt::Display for MixedSnapshotAcknowledgementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Decode(_) => "mixed snapshot acknowledgement decoding failed",
            Self::Acknowledgement(_) => "mixed snapshot acknowledgement original prefix rejected",
            Self::Payload(_) => "mixed snapshot acknowledgement complete carrier rejected",
        })
    }
}
impl Error for MixedSnapshotAcknowledgementError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decode(error) => Some(error),
            Self::Acknowledgement(error) => Some(error),
            Self::Payload(error) => Some(error),
        }
    }
}
