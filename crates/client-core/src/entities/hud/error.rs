// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::field_error::FieldReadError;
use crate::message_body::BodyReadError;
use std::{error::Error, fmt};

/// Fixed HUD-section failures without coordinates, colors, resource IDs or source bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HudError {
    /// The prior HUD state belongs to another explicit baseline profile.
    ReferenceProfile,
    /// The inclusive last-field index exceeds the reviewed forty-field catalog.
    FieldCount,
    /// A static field rule cannot be represented by the decoder.
    FieldEncoding,
    /// A compact float cannot convert the supplied prior value as required.
    ReferenceFloat,
    /// A changed legacy origin cannot use its prior coordinate.
    ReferenceCoordinate,
    /// Shared primitive reading failed before complete section publication.
    Body(BodyReadError),
}
impl fmt::Display for HudError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ReferenceProfile => "HUD reference profile mismatch",
            Self::FieldCount => "invalid HUD last-field index",
            Self::FieldEncoding => "unsupported static HUD field rule",
            Self::ReferenceFloat => "HUD compact float reference is invalid",
            Self::ReferenceCoordinate => "HUD legacy coordinate reference is invalid",
            Self::Body(_) => "invalid or truncated HUD section",
        })
    }
}
impl Error for HudError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            _ => None,
        }
    }
}
impl From<BodyReadError> for HudError {
    fn from(error: BodyReadError) -> Self {
        Self::Body(error)
    }
}
impl From<FieldReadError> for HudError {
    fn from(error: FieldReadError) -> Self {
        match error {
            FieldReadError::Body(error) => Self::Body(error),
            FieldReadError::Encoding => Self::FieldEncoding,
            FieldReadError::ReferenceFloat => Self::ReferenceFloat,
            FieldReadError::ReferenceCoordinate => Self::ReferenceCoordinate,
        }
    }
}
