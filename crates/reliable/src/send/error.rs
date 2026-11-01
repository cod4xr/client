// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed queue/transmission/ACK failures without retained bytes or routing values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendError {
    /// Invalid exact caller storage or explicit in-flight span.
    Storage,
    /// The complete enqueue operation cannot fit without overwriting retained data.
    Capacity,
    /// Further data would overflow signed cumulative ACK space; reconnect.
    SequenceExhausted,
    /// A sequence does not identify a currently queued original fragment.
    Missing,
    /// A queued sequence is beyond the current host-selected in-flight span.
    OutsideWindow,
    /// Peer selectively acknowledged this retained fragment; do not retransmit it.
    Selective,
    /// A cumulative ACK regressed below the already released base.
    AckRegression,
    /// A cumulative ACK exceeds queued data or the current in-flight span.
    AckFuture,
    /// An ACK claims a fragment that has never been admitted by actual successful I/O.
    AckUnsent,
    /// A selective range lies outside the complete queued/in-flight ACK domain.
    AckRange,
}

impl fmt::Display for SendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Storage => "invalid reliable send storage geometry",
            Self::Capacity => "reliable send queue capacity insufficient",
            Self::SequenceExhausted => "reliable send sequence exhausted; reconnect required",
            Self::Missing => "reliable fragment is not retained in the send queue",
            Self::OutsideWindow => "reliable fragment lies beyond the current send span",
            Self::Selective => "reliable fragment was selectively acknowledged",
            Self::AckRegression => "reliable cumulative acknowledgement regressed",
            Self::AckFuture => "reliable cumulative acknowledgement exceeds the send window",
            Self::AckUnsent => "reliable acknowledgement claims unsent data",
            Self::AckRange => "reliable selective acknowledgement exceeds retained send data",
        })
    }
}

impl Error for SendError {}
