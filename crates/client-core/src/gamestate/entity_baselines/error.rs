// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::message_body::BodyReadError;
use std::{error::Error, fmt};

/// Fixed entity-baseline failures, excluding raw entity state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntityBaselineError {
    /// This constructor/operation lacks the selected profile's explicit context.
    UnsupportedProfile,
    /// A map-center component cannot form a finite native signed-32 coordinate bias.
    InvalidMapCenter,
    /// A gamestate baseline cannot remove an entity from its zero reference.
    Removed,
    /// The last changed field is zero or outside the selected entity layout.
    FieldCount,
    /// A trusted static field catalog contains an unsupported encoding rule.
    FieldEncoding,
    /// Shared primitive/index reading failed without committing the decoder.
    Body(BodyReadError),
}

impl fmt::Display for EntityBaselineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedProfile => "entity baseline unsupported by selected profile",
            Self::InvalidMapCenter => "invalid legacy entity map-center component",
            Self::Removed => "gamestate entity baseline cannot be a removal",
            Self::FieldCount => "invalid entity baseline changed-field count",
            Self::FieldEncoding => "unsupported static entity baseline field rule",
            Self::Body(_) => "invalid or truncated entity baseline field",
        })
    }
}
impl Error for EntityBaselineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            _ => None,
        }
    }
}
impl From<BodyReadError> for EntityBaselineError {
    fn from(value: BodyReadError) -> Self {
        Self::Body(value)
    }
}
