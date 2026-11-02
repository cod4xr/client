// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::message_body::BodyReadError;
use std::{error::Error, fmt};

/// Bounded validation failures without private configstring contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigStringError {
    /// Count is nonpositive or exceeds the selected baseline namespace.
    Count,
    /// Explicit entries exceed the host inspection budget.
    RecordBudget,
    /// Caller metadata cannot retain all entries.
    RecordCapacity,
    /// An index is outside the profile or is not strictly increasing within the block.
    Index,
    /// A string exceeds the stricter native gamestate field bound.
    StringLength,
    /// Combined normalized text exceeds the host budget.
    TextBudget,
    /// Caller final text storage is too short.
    OutputCapacity,
    /// Shared primitive/string validation or private scratch capacity failed.
    Body(BodyReadError),
}

impl fmt::Display for ConfigStringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Count => "invalid gamestate configstring count",
            Self::RecordBudget => "gamestate configstring record budget exhausted",
            Self::RecordCapacity => "gamestate configstring metadata capacity insufficient",
            Self::Index => "invalid gamestate configstring index or order",
            Self::StringLength => "gamestate configstring field too long",
            Self::TextBudget => "gamestate configstring text budget exhausted",
            Self::OutputCapacity => "gamestate configstring output capacity insufficient",
            Self::Body(_) => "invalid or truncated gamestate configstring field",
        })
    }
}
impl Error for ConfigStringError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            _ => None,
        }
    }
}
impl From<BodyReadError> for ConfigStringError {
    fn from(value: BodyReadError) -> Self {
        Self::Body(value)
    }
}
