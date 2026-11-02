// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::message_body::BodyReadError;
use std::{error::Error, fmt};

/// Metadata-only array-section failures without inventory values or source bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerArrayError {
    /// The reference belongs to another explicitly selected baseline profile.
    ReferenceProfile,
    /// Shared primitive reading failed before section publication.
    Body(BodyReadError),
}
impl fmt::Display for PlayerArrayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ReferenceProfile => "player array reference profile mismatch",
            Self::Body(_) => "invalid or truncated player array section",
        })
    }
}
impl Error for PlayerArrayError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            Self::ReferenceProfile => None,
        }
    }
}
impl From<BodyReadError> for PlayerArrayError {
    fn from(error: BodyReadError) -> Self {
        Self::Body(error)
    }
}
