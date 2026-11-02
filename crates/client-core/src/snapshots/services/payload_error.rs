// SPDX-License-Identifier: AGPL-3.0-or-later
use super::SnapshotServiceError;
use crate::message_body::BodyReadError;
use std::{error::Error, fmt};

/// Actual decoded-payload binding failures without raw bytes or connection keys.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotPayloadError {
    /// Caller context disagrees with the original decoded-payload profile.
    Profile,
    /// Caller snapshot metadata disagrees with the actual channel message sequence.
    MessageSequence,
    /// The decoded body exceeds the shared ordinary profile's byte bound.
    Body(BodyReadError),
    /// Complete snapshot service validation or explicit prediction selection failed.
    Services(SnapshotServiceError),
}
impl fmt::Display for SnapshotPayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Profile => "snapshot context does not match decoded payload profile",
            Self::MessageSequence => "snapshot association does not match decoded payload sequence",
            Self::Body(_) => "invalid decoded snapshot body extent",
            Self::Services(_) => "invalid decoded snapshot services or prediction",
        })
    }
}
impl Error for SnapshotPayloadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            Self::Services(error) => Some(error),
            _ => None,
        }
    }
}
