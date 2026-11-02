// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::UserCommandEncodeError;
use std::{error::Error, fmt};

/// Fixed input-history failures without timestamps, input words or world labels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputHistoryError {
    /// No caller-funded input slots were provided.
    Storage,
    /// Every retained slot is still in use; retirement must be explicit.
    Capacity,
    /// Native input fields are invalid.
    Command(UserCommandEncodeError),
    /// Local identifiers exhausted their full sixty-four-bit range.
    SequenceExhausted,
    /// An identifier belongs to another host world/profile namespace.
    Binding,
    /// The exact requested input was retired or never retained.
    Missing,
    /// Batch endpoints do not form a positive increasing range.
    BatchRange,
    /// A batch exceeds the native per-packet command count.
    BatchCount,
    /// New transmission would skip an unsent input.
    TransmissionGap,
    /// Retirement regresses or includes inputs not yet transmitted.
    Retirement,
    /// Reset requires a strictly newer local world epoch.
    EpochOrder,
    /// Prediction report belongs to another profile.
    PredictionProfile,
    /// Snapshot belongs to another input world/profile namespace.
    SnapshotBinding,
}
impl fmt::Display for InputHistoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Storage => "input history requires caller storage",
            Self::Capacity => "input history capacity exhausted",
            Self::Command(_) => "invalid native input command",
            Self::SequenceExhausted => "input history local sequence exhausted",
            Self::Binding => "input history identifier binding mismatch",
            Self::Missing => "exact input command is not retained",
            Self::BatchRange => "invalid input batch range",
            Self::BatchCount => "input batch exceeds native packet count",
            Self::TransmissionGap => "input transmission would skip pending commands",
            Self::Retirement => "invalid input retirement range",
            Self::EpochOrder => "input history reset requires a newer epoch",
            Self::PredictionProfile => "input prediction profile mismatch",
            Self::SnapshotBinding => "input snapshot world or profile mismatch",
        })
    }
}
impl Error for InputHistoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Command(error) => Some(error),
            _ => None,
        }
    }
}
