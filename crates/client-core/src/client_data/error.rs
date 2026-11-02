// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::message_body::BodyReadError;
use std::{error::Error, fmt};

/// Fixed record validation errors, excluding private player text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientDataError {
    /// This explicitly selected baseline has no reviewed client-data record.
    UnsupportedProfile,
    /// Player identifier is outside the selected baseline namespace.
    ClientIndex,
    /// Shared field parsing, string bounds or native field capacity failed.
    Body(BodyReadError),
}

impl fmt::Display for ClientDataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedProfile => "client-data record unsupported by selected profile",
            Self::ClientIndex => "client-data player identifier outside baseline namespace",
            Self::Body(_) => "invalid or truncated client-data field",
        })
    }
}
impl Error for ClientDataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            _ => None,
        }
    }
}
impl From<BodyReadError> for ClientDataError {
    fn from(value: BodyReadError) -> Self {
        Self::Body(value)
    }
}
