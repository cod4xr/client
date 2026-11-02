// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::ServerPayloadError;
use crate::{commands::ClientCommandError, snapshots::services::SnapshotPayloadError};
use std::{error::Error, fmt};

/// Bounded original-owner snapshot/ACK preparation failures without private data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotAcknowledgementError {
    /// Shared header, retained-key transform or decompression failed.
    Decode(ServerPayloadError),
    /// The original owner's transmitted/retained ACK prefix cannot be prepared.
    Acknowledgement(ClientCommandError),
    /// Actual payload binding, service EOF or snapshot/prediction validation failed.
    Payload(SnapshotPayloadError),
}
impl fmt::Display for SnapshotAcknowledgementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Decode(_) => "snapshot acknowledgement payload decoding failed",
            Self::Acknowledgement(_) => "snapshot acknowledgement original prefix rejected",
            Self::Payload(_) => "snapshot acknowledgement complete carrier rejected",
        })
    }
}
impl Error for SnapshotAcknowledgementError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decode(error) => Some(error),
            Self::Acknowledgement(error) => Some(error),
            Self::Payload(error) => Some(error),
        }
    }
}
