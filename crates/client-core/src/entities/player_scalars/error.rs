// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::field_error::FieldReadError;
use crate::message_body::BodyReadError;
use std::{error::Error, fmt};

/// Metadata-only prefix failures without positions, inventory bits or source bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerScalarError {
    /// The prior scalar projection belongs to another explicit baseline profile.
    ReferenceProfile,
    /// The scalar field-prefix count exceeds the reviewed 141-field catalog.
    FieldCount,
    /// A static field rule is outside this reviewed live scalar schema.
    FieldEncoding,
    /// An ordinary compact float cannot use the supplied prior integer component.
    ReferenceFloat,
    /// Shared primitive reading failed before whole-prefix publication.
    Body(BodyReadError),
}
impl fmt::Display for PlayerScalarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ReferenceProfile => "player scalar reference profile mismatch",
            Self::FieldCount => "invalid player scalar field count",
            Self::FieldEncoding => "unsupported static player scalar field rule",
            Self::ReferenceFloat => "player scalar compact float reference is invalid",
            Self::Body(_) => "invalid or truncated player scalar prefix",
        })
    }
}
impl Error for PlayerScalarError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            _ => None,
        }
    }
}
impl From<BodyReadError> for PlayerScalarError {
    fn from(error: BodyReadError) -> Self {
        Self::Body(error)
    }
}
impl From<FieldReadError> for PlayerScalarError {
    fn from(error: FieldReadError) -> Self {
        match error {
            FieldReadError::Body(error) => Self::Body(error),
            FieldReadError::ReferenceFloat => Self::ReferenceFloat,
            FieldReadError::Encoding | FieldReadError::ReferenceCoordinate => Self::FieldEncoding,
        }
    }
}
