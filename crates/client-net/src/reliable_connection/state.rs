// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    fmt,
    time::{Duration, Instant},
};

use super::{OrdinaryConnectionSender, ReliableConnectionStats, ReliableUsage};
use crate::{ChannelContext, ClientChannel, NetworkError};
use cod4xr_client_core::server_payload::ServerPayloadDecoder;
use cod4xr_reliable::{
    endpoint::{DrainRun, EndpointConfig, EndpointError, EndpointStorage, ReliableEndpoint},
    framing::ReliableFrame,
    send::QueuedFragments,
};

/// Original acknowledged socket, one clock origin and one fresh endpoint.
/// No detached endpoint/window/clock access can mix state from another connection.
pub struct ReliableConnection<'storage> {
    pub(super) channel: ClientChannel<'storage>,
    pub(super) endpoint: ReliableEndpoint<'storage>,
    pub(super) started: Instant,
    pub(super) usage: ReliableUsage,
}

impl<'storage> ReliableConnection<'storage> {
    pub(crate) fn new(
        channel: ClientChannel<'storage>,
        storage: EndpointStorage<'storage>,
        config: EndpointConfig,
    ) -> Result<Self, NetworkError> {
        let endpoint = ReliableEndpoint::new(Duration::ZERO, config, storage)
            .map_err(NetworkError::Endpoint)?;
        Ok(Self {
            channel,
            endpoint,
            started: Instant::now(),
            usage: ReliableUsage::default(),
        })
    }

    /// Borrows the original immutable connection context; raw fields remain private data.
    #[must_use]
    pub fn context(&self) -> &ChannelContext {
        self.channel.context()
    }

    /// Creates the original reviewed ordinary-message decoder without reading a packet.
    ///
    /// # Errors
    /// Propagates unsupported original-profile errors without selecting a fallback.
    pub fn server_decoder(&self) -> Result<ServerPayloadDecoder, NetworkError> {
        self.channel.server_decoder()
    }

    /// Queues one complete opaque application body within the original endpoint budget.
    ///
    /// # Errors
    /// Rejects terminal framing failure, queue capacity and sequence/length errors.
    pub fn enqueue(&mut self, body: &[u8]) -> Result<QueuedFragments, NetworkError> {
        self.ensure_active()?;
        self.endpoint.enqueue(body).map_err(NetworkError::Endpoint)
    }

    /// Borrows the single serialized ordinary sender. Finish/drop its transmission before
    /// servicing another operation; no independently concurrent socket writers are created.
    ///
    /// # Errors
    /// Rejects a terminally failed endpoint before exposing I/O.
    pub fn ordinary_sender(&mut self) -> Result<OrdinaryConnectionSender<'_>, NetworkError> {
        self.ensure_active()?;
        Ok(OrdinaryConnectionSender::new(self.channel.split().sender))
    }

    /// Drains at most the caller byte/inspection quota toward one complete frame.
    /// Work counters survive errors; a terminal frame prefix prevents subsequent network I/O.
    #[must_use]
    pub fn drain(
        &mut self,
        scratch: &mut [u8],
        max_bytes: usize,
        max_fragments: usize,
    ) -> DrainRun {
        let was_failed = self.endpoint.state().failed;
        let run = self.endpoint.drain(scratch, max_bytes, max_fragments);
        self.usage.drained_bytes = self
            .usage
            .drained_bytes
            .saturating_add(u64::try_from(run.usage.bytes).unwrap_or(u64::MAX));
        self.usage.fragment_inspections = self
            .usage
            .fragment_inspections
            .saturating_add(u64::try_from(run.usage.examined).unwrap_or(u64::MAX));
        if !was_failed && run.outcome.is_err() {
            self.usage.framing_failures = self.usage.framing_failures.saturating_add(1);
        }
        run
    }

    /// Borrows a complete opaque body. Typed application validation must precede its use.
    #[must_use]
    pub fn frame(&self) -> Option<ReliableFrame<'_>> {
        self.endpoint.frame()
    }

    /// Releases one host-inspected complete frame and records that explicit consumption.
    #[must_use]
    pub fn consume_frame(&mut self) -> bool {
        let consumed = self.endpoint.consume_frame();
        if consumed {
            self.usage.released_frames = self.usage.released_frames.saturating_add(1);
        }
        consumed
    }

    /// Returns counts/readiness only, excluding payloads, routing identifiers and cookies.
    #[must_use]
    pub fn stats(&self) -> ReliableConnectionStats {
        ReliableConnectionStats {
            send: self.channel.send_usage(),
            receive: self.channel.receive_usage(),
            reliable: self.usage,
            endpoint: self.endpoint.state(),
        }
    }

    pub(super) fn ensure_active(&self) -> Result<(), NetworkError> {
        if self.endpoint.state().failed {
            return Err(NetworkError::Endpoint(EndpointError::Failed));
        }
        Ok(())
    }
}

impl fmt::Debug for ReliableConnection<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReliableConnection")
            .field("stats", &self.stats())
            .finish_non_exhaustive()
    }
}
