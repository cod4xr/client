// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed index-build failures without keys, payloads or archive identifiers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MountIndexError {
    /// Input candidate count exceeds the host inspection budget.
    CandidateBudget,
    /// Combined input key bytes exceed the host inspection budget or overflow.
    InputKeyBudget,
    /// Private sorting scratch cannot retain all input indices.
    ScratchCapacity,
    /// Retained unique files exceed the host retention budget.
    FileBudget,
    /// Retained key bytes exceed the host retention budget or overflow.
    KeyBudget,
    /// Final descriptors cannot retain all unique files.
    SlotCapacity,
    /// Final text cannot retain all canonical key bytes.
    OutputCapacity,
}
impl fmt::Display for MountIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::CandidateBudget => "mount-index candidate budget exhausted",
            Self::InputKeyBudget => "mount-index input key budget exhausted",
            Self::ScratchCapacity => "mount-index private scratch capacity insufficient",
            Self::FileBudget => "mount-index retained file budget exhausted",
            Self::KeyBudget => "mount-index retained key budget exhausted",
            Self::SlotCapacity => "mount-index final descriptor capacity insufficient",
            Self::OutputCapacity => "mount-index final key capacity insufficient",
        })
    }
}
impl Error for MountIndexError {}
