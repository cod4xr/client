// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::framing::FrameError;
use std::{error::Error, fmt};

/// Fixed queue/transmission/ACK failures without retained bytes or routing values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendError {
    /// A complete application frame could not form its length prefix.
    Frame(FrameError),
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
        match self {
            Self::Frame(error) => error.fmt(formatter),
            Self::Storage => formatter.write_str("invalid reliable send storage geometry"),
            Self::Capacity => formatter.write_str("reliable send queue capacity insufficient"),
            Self::SequenceExhausted => {
                formatter.write_str("reliable send sequence exhausted; reconnect required")
            }
            Self::Missing => {
                formatter.write_str("reliable fragment is not retained in the send queue")
            }
            Self::OutsideWindow => {
                formatter.write_str("reliable fragment lies beyond the current send span")
            }
            Self::Selective => {
                formatter.write_str("reliable fragment was selectively acknowledged")
            }
            Self::AckRegression => {
                formatter.write_str("reliable cumulative acknowledgement regressed")
            }
            Self::AckFuture => {
                formatter.write_str("reliable cumulative acknowledgement exceeds the send window")
            }
            Self::AckUnsent => formatter.write_str("reliable acknowledgement claims unsent data"),
            Self::AckRange => {
                formatter.write_str("reliable selective acknowledgement exceeds retained send data")
            }
        }
    }
}

impl Error for SendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::Frame(error) = self {
            Some(error)
        } else {
            None
        }
    }
}
