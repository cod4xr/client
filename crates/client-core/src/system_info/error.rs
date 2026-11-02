// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

use cod4x_protocol::InfoError;

/// Fixed system-info failures without remote keys, values or identifiers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemInfoError {
    /// Effective config state does not contain native system info.
    MissingSystemInfo,
    /// Encoded text exceeds the host's inspection budget.
    ByteBudget,
    /// Field inspection exceeds the host's budget.
    FieldBudget,
    /// Shared big-info syntax or wire length validation failed.
    Info(InfoError),
    /// The required server identifier is absent.
    MissingServerId,
    /// The critical identifier occurs more than once, including different ASCII case.
    DuplicateServerId,
    /// The identifier is not a complete decimal signed-32 or unsigned-32 representation.
    InvalidServerId,
}

impl fmt::Display for SystemInfoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingSystemInfo => "native system-info configstring missing",
            Self::ByteBudget => "system-info byte budget exhausted",
            Self::FieldBudget => "system-info field budget exhausted",
            Self::Info(_) => "system-info shared validation failed",
            Self::MissingServerId => "system-info server identifier missing",
            Self::DuplicateServerId => "system-info server identifier ambiguous",
            Self::InvalidServerId => "system-info server identifier invalid",
        })
    }
}

impl Error for SystemInfoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Info(error) => Some(error),
            _ => None,
        }
    }
}
