// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, time::Instant};

use crate::SentReliableDatagram;
use cod4xr_reliable::{
    ReliableError,
    endpoint::{EndpointError, IngestReport},
};

/// One send operation, without an internal retry/burst loop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionSend {
    /// No datagram is due before this time or the caller's earlier deadline.
    WaitUntil(Instant),
    /// One complete datagram was accepted by the OS.
    Sent(SentReliableDatagram),
}

/// Fixed recoverable reliable-packet rejection, excluding peer data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReliableRejection {
    /// Full body syntax/size validation failed.
    Body(ReliableError),
    /// Receive or outgoing ACK admission failed without changing endpoint state.
    State(EndpointError),
}

/// One received packet. Other traffic stays inside the host's scoped callback.
pub enum ConnectionReceive<T> {
    /// One complete reliable packet committed atomically.
    Accepted(IngestReport),
    /// Invalid reliable body/window data, without partial state admission.
    Rejected(ReliableRejection),
    /// Host result for ordinary/channel-drop/non-reliable auxiliary traffic.
    Other(T),
}

impl<T> fmt::Debug for ConnectionReceive<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Accepted(report) => formatter.debug_tuple("Accepted").field(report).finish(),
            Self::Rejected(reason) => formatter.debug_tuple("Rejected").field(reason).finish(),
            Self::Other(_) => formatter.write_str("Other(..)"),
        }
    }
}
