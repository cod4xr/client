// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::SnapshotError;
use crate::message_body::BodyReadError;
use std::{error::Error, fmt};

/// Fixed-category service framing failures without raw snapshot data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotServiceError {
    /// A zero record budget cannot admit even one service.
    InvalidBudget,
    /// Inspection exhausted before complete snapshot/EOF validation.
    RecordBudget,
    /// Input ended before the native logical EOF service.
    MissingEof,
    /// Logical EOF arrived without a snapshot service.
    MissingSnapshot,
    /// A service requires a different combined decoder; its body was never skipped.
    UnsupportedService(u8),
    /// Whole-snapshot decoding or prediction resolution failed.
    Snapshot(SnapshotError),
    /// Shared primitive reading failed.
    Body(BodyReadError),
}
impl fmt::Display for SnapshotServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidBudget => "invalid snapshot service inspection budget",
            Self::RecordBudget => "snapshot service inspection budget exhausted",
            Self::MissingEof => "snapshot service logical EOF missing",
            Self::MissingSnapshot => "snapshot service required before EOF",
            Self::UnsupportedService(_) => "service requires a combined decoder",
            Self::Snapshot(_) => "invalid snapshot or prediction in service carrier",
            Self::Body(_) => "invalid or truncated snapshot service framing",
        })
    }
}
impl Error for SnapshotServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Snapshot(error) => Some(error),
            Self::Body(error) => Some(error),
            _ => None,
        }
    }
}
