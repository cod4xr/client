// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed prediction-retention failures without timestamps or player words.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredictionHistoryError {
    /// No caller-funded lookup cells were supplied.
    Storage,
    /// The explicit world label differs from the current owner.
    Epoch,
    /// The sample/request profile differs from the current owner.
    Profile,
    /// No free cell remains; no implicit eviction occurs.
    Capacity,
    /// Insertion would silently replace an existing exact-time sample.
    Duplicate,
    /// The exact requested native time has not been retained.
    Missing,
    /// Reset requires a strictly newer world label.
    EpochOrder,
}
impl fmt::Display for PredictionHistoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Storage => "prediction history requires storage",
            Self::Epoch => "prediction history world differs",
            Self::Profile => "prediction history profile differs",
            Self::Capacity => "prediction history capacity exhausted",
            Self::Duplicate => "prediction sample already retained",
            Self::Missing => "exact prediction sample is missing",
            Self::EpochOrder => "prediction history epoch must advance",
        })
    }
}
impl Error for PredictionHistoryError {}
