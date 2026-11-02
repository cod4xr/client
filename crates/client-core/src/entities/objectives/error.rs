// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::field_error::FieldReadError;
use crate::message_body::BodyReadError;
use std::{error::Error, fmt};

/// Fixed objective-section failures without coordinates, identifiers or source bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectiveError {
    /// The prior set belongs to another explicit baseline profile.
    ReferenceProfile,
    /// A compact float delta cannot use the supplied prior integer component.
    ReferenceFloat,
    /// A static value rule lies outside this reviewed objective schema.
    FieldEncoding,
    /// Shared primitive reading failed before section publication.
    Body(BodyReadError),
}
impl fmt::Display for ObjectiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ReferenceProfile => "objective reference profile mismatch",
            Self::ReferenceFloat => "objective compact float reference is invalid",
            Self::FieldEncoding => "unsupported static objective field rule",
            Self::Body(_) => "invalid or truncated objective section",
        })
    }
}
impl Error for ObjectiveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            _ => None,
        }
    }
}
impl From<BodyReadError> for ObjectiveError {
    fn from(error: BodyReadError) -> Self {
        Self::Body(error)
    }
}
impl From<FieldReadError> for ObjectiveError {
    fn from(error: FieldReadError) -> Self {
        match error {
            FieldReadError::Body(error) => Self::Body(error),
            FieldReadError::ReferenceFloat => Self::ReferenceFloat,
            FieldReadError::Encoding | FieldReadError::ReferenceCoordinate => Self::FieldEncoding,
        }
    }
}
