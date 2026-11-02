// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed sparse-merge failures, excluding source values and peer text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigStateError {
    /// Combined source record inspection exceeds the host budget.
    InputRecordBudget,
    /// Source byte inspection exceeds the host budget or overflows the host.
    InputByteBudget,
    /// Caller private descriptor scratch cannot retain the source records.
    ScratchCapacity,
    /// Default IDs are duplicated or not strictly increasing.
    DefaultOrder,
    /// A nonempty patch stream lacks its initial native block marker.
    MissingBlockStart,
    /// Explicit IDs duplicate or descend within one block.
    PatchOrder,
    /// A source value contains an embedded NUL outside configstring semantics.
    EmbeddedNul,
    /// Unique effective records exceed the host retention budget.
    RecordBudget,
    /// Effective text exceeds the host budget or overflows the host.
    TextBudget,
    /// Caller final metadata cannot retain all effective records.
    RecordCapacity,
    /// Caller final text cannot retain all effective values.
    OutputCapacity,
}
impl fmt::Display for ConfigStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InputRecordBudget => "config-state input record budget exhausted",
            Self::InputByteBudget => "config-state input byte budget exhausted",
            Self::ScratchCapacity => "config-state private scratch capacity insufficient",
            Self::DefaultOrder => "config-state default identifiers not strictly increasing",
            Self::MissingBlockStart => "config-state initial block marker missing",
            Self::PatchOrder => "config-state patch identifiers not increasing within block",
            Self::EmbeddedNul => "config-state value contains an embedded NUL",
            Self::RecordBudget => "config-state effective record budget exhausted",
            Self::TextBudget => "config-state effective text budget exhausted",
            Self::RecordCapacity => "config-state final metadata capacity insufficient",
            Self::OutputCapacity => "config-state final text capacity insufficient",
        })
    }
}
impl Error for ConfigStateError {}
