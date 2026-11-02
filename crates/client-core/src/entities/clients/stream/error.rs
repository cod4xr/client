// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::ClientStateError;
use std::{error::Error, fmt};

/// Fixed substream failures without raw client names, resource IDs or packet bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientStreamError {
    /// The complete previous source exceeds inspection policy.
    PreviousBudget,
    /// The prior source is unordered, duplicated, uninitialized or outside six bits.
    Source,
    /// Record-header work exhausted before the separate end bit.
    RecordBudget,
    /// Complete merged retention exceeds host policy.
    ClientBudget,
    /// Final caller cells cannot hold the complete merged result.
    OutputCapacity,
    /// An individual record or its framing/index failed before publication.
    Delta(ClientStateError),
}
impl fmt::Display for ClientStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::PreviousBudget => "snapshot client prior-source budget exhausted",
            Self::Source => "invalid ordered snapshot client source",
            Self::RecordBudget => "snapshot client wire-record budget exhausted",
            Self::ClientBudget => "snapshot client output budget exhausted",
            Self::OutputCapacity => "snapshot client final capacity insufficient",
            Self::Delta(_) => "invalid or truncated snapshot client record",
        })
    }
}
impl Error for ClientStreamError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Delta(error) => Some(error),
            _ => None,
        }
    }
}
impl From<ClientStateError> for ClientStreamError {
    fn from(error: ClientStateError) -> Self {
        Self::Delta(error)
    }
}
