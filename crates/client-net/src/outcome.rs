// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    fmt,
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use cod4xr_client_core::connect::{ConnectObservation, ConnectReplyKind, ConnectUsage};

use crate::{NetworkError, peer::PeerIo};

/// Owned terminal metadata. Remote payloads have no command/filesystem authority.
pub struct ConnectSummary {
    peer: SocketAddr,
    protocol: i32,
    qport: u16,
    kind: ConnectReplyKind,
    payload: Vec<u8>,
    elapsed: Duration,
    usage: ConnectUsage,
}

impl ConnectSummary {
    pub(crate) fn capture(
        observation: ConnectObservation<'_>,
        storage: &mut Vec<u8>,
    ) -> Result<Self, NetworkError> {
        if observation.payload().len() > storage.capacity() {
            return Err(NetworkError::ReplyCapacity);
        }
        storage.clear();
        storage.extend_from_slice(observation.payload());
        Ok(Self {
            peer: observation.peer(),
            protocol: observation.requested_protocol(),
            qport: observation.qport(),
            kind: observation.kind(),
            payload: std::mem::take(storage),
            elapsed: observation.elapsed(),
            usage: observation.usage(),
        })
    }

    /// Returns the selected endpoint, without accepting redirection.
    #[must_use]
    pub fn peer(&self) -> SocketAddr {
        self.peer
    }
    /// Returns the original explicit protocol, not a negotiated replacement.
    #[must_use]
    pub fn requested_protocol(&self) -> i32 {
        self.protocol
    }
    /// Returns the original routing value needed for channel setup.
    #[must_use]
    pub fn qport(&self) -> u16 {
        self.qport
    }
    /// Returns acknowledgement or refusal; never an active gameplay state.
    #[must_use]
    pub fn kind(&self) -> ConnectReplyKind {
        self.kind
    }
    /// Borrows untrusted mod-label/refusal bytes; never execute, open or log them.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    /// Returns elapsed connect-phase time, including all retries.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
    /// Returns all work charged before the terminal observation.
    #[must_use]
    pub fn usage(&self) -> ConnectUsage {
        self.usage
    }
}

impl fmt::Debug for ConnectSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConnectSummary")
            .field("kind", &self.kind)
            .field("elapsed", &self.elapsed)
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}

/// Owns the acknowledged socket; channel and gamestate setup are still required.
pub struct AcknowledgedSocket {
    io: PeerIo,
    summary: ConnectSummary,
}

impl AcknowledgedSocket {
    pub(crate) fn new(io: PeerIo, summary: ConnectSummary) -> Self {
        Self { io, summary }
    }
    /// Borrows terminal metadata without releasing transport ownership.
    #[must_use]
    pub fn summary(&self) -> &ConnectSummary {
        &self.summary
    }
    /// Transfers the original selected socket and metadata to a channel owner.
    /// The socket remains blocking with finite read/write timeouts; the new owner
    /// must configure its own I/O policy. This does not perform channel setup.
    #[must_use]
    pub fn into_parts(self) -> (UdpSocket, ConnectSummary) {
        (self.io.socket, self.summary)
    }
}

impl fmt::Debug for AcknowledgedSocket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AcknowledgedSocket")
            .field("summary", &self.summary)
            .finish_non_exhaustive()
    }
}

/// One terminal no-auth connect response, without implying playable admission.
#[derive(Debug)]
pub enum ConnectOutcome {
    /// Selected peer acknowledged; the original socket is retained.
    Acknowledged(AcknowledgedSocket),
    /// Peer refused; socket was closed and display bytes remain inert data.
    Refused(ConnectSummary),
}
