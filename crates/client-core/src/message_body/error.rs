// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

use cod4x_protocol::{LegacyIndexError, MessageError};

/// Fixed body-view errors, excluding input bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyReadError {
    /// Shared primitive validation or the explicit whole-body host budget failed.
    Message(MessageError),
    /// Shared legacy index ordering/namespace validation failed.
    Index(LegacyIndexError),
    /// A private shared-reader view violated the forward cursor/cache contract.
    Cursor,
}

impl fmt::Display for BodyReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(error) => error.fmt(formatter),
            Self::Index(error) => error.fmt(formatter),
            Self::Cursor => formatter.write_str("shared body view produced an invalid cursor"),
        }
    }
}
impl Error for BodyReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Message(error) => Some(error),
            Self::Index(error) => Some(error),
            Self::Cursor => None,
        }
    }
}
impl From<MessageError> for BodyReadError {
    fn from(value: MessageError) -> Self {
        Self::Message(value)
    }
}
