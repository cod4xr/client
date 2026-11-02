// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::field_error::FieldReadError;
use crate::message_body::BodyReadError;
use std::{error::Error, fmt};

/// Fixed delta-record failures without state words, coordinates or source bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntityDeltaError {
    /// Only the reviewed protocol-21 coordinate profile is supported here.
    UnsupportedProfile,
    /// The supplied prior state belongs to another entity number.
    ReferenceNumber,
    /// The changed-field count is zero or exceeds the selected target layout.
    FieldCount,
    /// A static field encoding cannot be represented by this decoder.
    FieldEncoding,
    /// A compact float delta has a nonfinite/out-of-range prior integer component.
    /// The reviewed writer uses its raw form for these prior states.
    ReferenceFloat,
    /// Shared primitive/index reading failed before record commit.
    Body(BodyReadError),
}
impl fmt::Display for EntityDeltaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnsupportedProfile => "entity delta requires the reviewed protocol-21 profile",
            Self::ReferenceNumber => "entity delta reference number mismatch",
            Self::FieldCount => "invalid entity delta changed-field count",
            Self::FieldEncoding => "unsupported static entity delta field rule",
            Self::ReferenceFloat => "entity delta compact float reference is invalid",
            Self::Body(_) => "invalid or truncated entity delta record",
        })
    }
}
impl Error for EntityDeltaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            _ => None,
        }
    }
}
impl From<BodyReadError> for EntityDeltaError {
    fn from(error: BodyReadError) -> Self {
        Self::Body(error)
    }
}
impl From<FieldReadError> for EntityDeltaError {
    fn from(error: FieldReadError) -> Self {
        match error {
            FieldReadError::Body(error) => Self::Body(error),
            FieldReadError::Encoding => Self::FieldEncoding,
            FieldReadError::ReferenceFloat => Self::ReferenceFloat,
        }
    }
}
