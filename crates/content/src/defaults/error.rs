// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::pe32::Pe32Error;
use std::{error::Error, fmt};

/// Fixed import failures without game data, file paths or computed unknown hashes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DefaultsError {
    /// The complete caller file is not a hash-reviewed image profile.
    UnknownImage,
    /// Native table structure/count/order/reserved fields differ from its reviewed layout.
    Layout,
    /// Record retention exceeds the host budget.
    RecordBudget,
    /// Combined raw text exceeds the host budget.
    TextBudget,
    /// A value lacks an in-section terminator within the host per-value bound.
    ValueBudget,
    /// Caller metadata cannot retain the whole table.
    RecordCapacity,
    /// Caller text cannot retain the whole table.
    OutputCapacity,
    /// PE metadata, file budget or address lookup failed.
    Image(Pe32Error),
}
impl fmt::Display for DefaultsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnknownImage => "client image has no reviewed data profile",
            Self::Layout => "client default table differs from reviewed layout",
            Self::RecordBudget => "client default record budget exhausted",
            Self::TextBudget => "client default text budget exhausted",
            Self::ValueBudget => "client default terminator or value budget invalid",
            Self::RecordCapacity => "client default metadata capacity insufficient",
            Self::OutputCapacity => "client default output capacity insufficient",
            Self::Image(_) => "client image metadata or file-backed address invalid",
        })
    }
}
impl Error for DefaultsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Image(error) => Some(error),
            _ => None,
        }
    }
}
impl From<Pe32Error> for DefaultsError {
    fn from(value: Pe32Error) -> Self {
        Self::Image(value)
    }
}
