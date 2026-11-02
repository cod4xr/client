// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed preparation failures without command text, map names or disconnect reasons.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreGameError {
    /// Total bytes in all newly admitted commands exceed host inspection policy.
    CommandByteBudget,
    /// Recognized pre-game notices exceed host publication policy.
    EventBudget,
}
impl fmt::Display for PreGameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::CommandByteBudget => "pre-game command byte budget exhausted",
            Self::EventBudget => "pre-game notice budget exhausted",
        })
    }
}
impl Error for PreGameError {}
