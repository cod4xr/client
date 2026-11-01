// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed receive-window failures without fragment data or private routing metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiveError {
    /// Caller byte/metadata geometry cannot hold the selected representable span.
    Storage,
    /// A data fragment lies beyond the caller's current receive span.
    OutOfWindow,
    /// Consuming this sequence would exhaust signed cumulative ACK space; reconnect.
    SequenceExhausted,
}

impl fmt::Display for ReceiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Storage => "invalid reliable receive storage geometry",
            Self::OutOfWindow => "reliable fragment lies outside the receive window",
            Self::SequenceExhausted => "reliable receive sequence exhausted; reconnect required",
        })
    }
}

impl Error for ReceiveError {}
