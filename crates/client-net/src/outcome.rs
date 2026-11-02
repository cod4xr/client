// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    fmt,
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use cod4xr_client_core::connect::{ConnectObservation, ConnectReplyKind, ConnectUsage};

use crate::{ChannelContext, ClientChannel, NetworkError, ReliableConnection, peer::PeerIo};

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
    context: ChannelContext,
}

impl AcknowledgedSocket {
    pub(crate) fn new(io: PeerIo, summary: ConnectSummary, challenge: i32) -> Self {
        Self {
            io,
            context: ChannelContext::new(summary, challenge),
        }
    }
    /// Borrows terminal metadata without releasing transport ownership.
    #[must_use]
    pub fn summary(&self) -> &ConnectSummary {
        self.context.summary()
    }
    /// Transfers the original selected socket and metadata to a channel owner.
    /// The socket remains blocking with finite read/write timeouts; the new owner
    /// must configure its own I/O policy. This does not perform channel setup.
    #[must_use]
    pub fn into_parts(self) -> (UdpSocket, ChannelContext) {
        (self.io.socket, self.context)
    }

    /// Consumes this socket into an ordinary channel using caller-owned reassembly storage.
    /// Message payload codecs, auxiliary reliable transport and gameplay remain separate.
    ///
    /// # Errors
    /// Rejects reassembly capacity above the shared logical-message profile.
    pub fn into_channel(self, storage: &mut [u8]) -> Result<ClientChannel<'_>, NetworkError> {
        ClientChannel::new(self.io, self.context, storage)
    }

    /// Consumes this original socket into a fresh reliable connection owner.
    /// All clocks, windows, routing and frame budgets stay tied to this connection.
    ///
    /// # Errors
    /// Rejects unsupported original profiles and invalid channel/endpoint storage.
    pub fn into_reliable<'storage>(
        self,
        reassembly: &'storage mut [u8],
        storage: cod4xr_reliable::endpoint::EndpointStorage<'storage>,
        config: cod4xr_reliable::endpoint::EndpointConfig,
    ) -> Result<ReliableConnection<'storage>, NetworkError> {
        if cod4xr_client_core::BaselineProtocol::from_number(self.summary().requested_protocol())
            != Some(cod4xr_client_core::BaselineProtocol::Cod4x21)
        {
            return Err(NetworkError::AuxiliaryProfile);
        }
        ReliableConnection::new(self.into_channel(reassembly)?, storage, config)
    }
}

impl fmt::Debug for AcknowledgedSocket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AcknowledgedSocket")
            .field("context", &self.context)
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
