// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::field_error::FieldReadError;
use crate::message_body::BodyReadError;
use std::{error::Error, fmt};

/// Metadata-only client-state failures; no remote names or state words are retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientStateError {
    /// This stream already consumed its end bit; reset before reading another stream.
    StreamEnded,
    /// The selected prior state belongs to another client index.
    ReferenceNumber,
    /// The prefix count exceeds the reviewed twenty-four-field schema.
    FieldCount,
    /// A static value rule is unsupported by the reviewed client-state schema.
    FieldEncoding,
    /// A compact float form cannot use the supplied prior integer component.
    ReferenceFloat,
    /// Shared index or primitive decoding failed before record commit.
    Body(BodyReadError),
}
impl fmt::Display for ClientStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::StreamEnded => "client-state stream already ended",
            Self::ReferenceNumber => "client-state reference number mismatch",
            Self::FieldCount => "invalid client-state field count",
            Self::FieldEncoding => "unsupported static client-state field rule",
            Self::ReferenceFloat => "client-state compact float reference is invalid",
            Self::Body(_) => "invalid or truncated client-state record",
        })
    }
}
impl Error for ClientStateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            _ => None,
        }
    }
}
impl From<BodyReadError> for ClientStateError {
    fn from(error: BodyReadError) -> Self {
        Self::Body(error)
    }
}
impl From<FieldReadError> for ClientStateError {
    fn from(error: FieldReadError) -> Self {
        match error {
            FieldReadError::Body(error) => Self::Body(error),
            FieldReadError::ReferenceFloat => Self::ReferenceFloat,
            FieldReadError::Encoding | FieldReadError::ReferenceCoordinate => Self::FieldEncoding,
        }
    }
}
