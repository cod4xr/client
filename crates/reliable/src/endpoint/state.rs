// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, time::Duration};

use super::{EndpointConfig, EndpointError, EndpointStorage};
use crate::{
    framing::{FrameAssembler, ReliableFrame, required_frame_bytes},
    receive::ReceiveWindow,
    schedule::{ReliableScheduler, SchedulePoll},
    send::{QueuedFragments, SendWindow},
};

/// Copyable resource/state counters without private bytes, routing or callback authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EndpointState {
    /// Retained outgoing fragments, including selectively acknowledged data.
    pub outgoing_fragments: usize,
    /// Retained original outgoing bytes.
    pub outgoing_bytes: usize,
    /// Consumed outgoing cumulative base.
    pub outgoing_acknowledge: u32,
    /// Next outgoing queue sequence.
    pub next_outgoing_sequence: u32,
    /// Retained incoming fragment slots, including a partially read head.
    pub incoming_fragments: usize,
    /// Unread incoming stream bytes, including data beyond gaps.
    pub incoming_bytes: usize,
    /// Fully consumed incoming cumulative base.
    pub incoming_acknowledge: u32,
    /// A complete application body awaits host validation/consumption.
    pub frame_ready: bool,
    /// An invalid frame prefix permanently failed this endpoint.
    pub failed: bool,
}

/// Unified reliable transport/framing state for one fresh host connection.
/// Peers/profiles, OS I/O, application decoding and session lifetime remain outside.
pub struct ReliableEndpoint<'buffers> {
    pub(super) outgoing: SendWindow<'buffers>,
    pub(super) incoming: ReceiveWindow<'buffers>,
    pub(super) scheduler: ReliableScheduler,
    pub(super) frames: FrameAssembler<'buffers>,
}

impl<'buffers> ReliableEndpoint<'buffers> {
    /// Preflights every buffer/timer before resetting either metadata array.
    /// This allocates nothing and never exposes dirty backing bytes.
    ///
    /// # Errors
    /// Rejects any invalid geometry, body representation or initial schedule atomically.
    pub fn new(
        now: Duration,
        config: EndpointConfig,
        storage: EndpointStorage<'buffers>,
    ) -> Result<Self, EndpointError> {
        let EndpointStorage {
            send_bytes,
            send_slots,
            receive_bytes,
            receive_slots,
            body,
        } = storage;
        SendWindow::validate_geometry(send_bytes.len(), send_slots.len(), config.send_window)
            .map_err(EndpointError::Send)?;
        ReceiveWindow::validate_geometry(receive_bytes.len(), receive_slots.len())
            .map_err(EndpointError::Receive)?;
        required_frame_bytes(body.len()).map_err(EndpointError::Frame)?;
        let scheduler =
            ReliableScheduler::new(now, config.schedule).map_err(EndpointError::Schedule)?;
        let frames = FrameAssembler::new(body).map_err(EndpointError::Frame)?;
        let outgoing = SendWindow::new(send_bytes, send_slots, config.send_window)
            .map_err(EndpointError::Send)?;
        let incoming =
            ReceiveWindow::new(receive_bytes, receive_slots).map_err(EndpointError::Receive)?;
        Ok(Self {
            outgoing,
            incoming,
            scheduler,
            frames,
        })
    }

    /// Queues one complete application frame atomically, without a temporary full-frame copy.
    /// Body bytes remain opaque to transport and grant no command authority.
    ///
    /// # Errors
    /// Rejects a failed endpoint, frame representation and complete outgoing capacity/sequence.
    pub fn enqueue(&mut self, body: &[u8]) -> Result<QueuedFragments, EndpointError> {
        self.ensure_active()?;
        let frame = ReliableFrame::new(body).map_err(EndpointError::Frame)?;
        self.outgoing
            .enqueue_frame(frame)
            .map_err(EndpointError::Send)
    }

    /// Selects a paced datagram over this endpoint's actual send/receive windows.
    ///
    /// # Errors
    /// Rejects a failed endpoint, backwards time, timer overflow and window selection failure.
    pub fn poll(&mut self, now: Duration) -> Result<SchedulePoll<'_, '_, 'buffers>, EndpointError> {
        self.ensure_active()?;
        self.scheduler
            .poll(now, &mut self.outgoing, &self.incoming)
            .map_err(EndpointError::Schedule)
    }

    /// Borrows only a complete opaque body. Host validation must precede application use.
    #[must_use]
    pub fn frame(&self) -> Option<ReliableFrame<'_>> {
        self.frames.frame()
    }

    /// Releases one host-inspected complete frame. Failed/incomplete frames remain untouched.
    #[must_use]
    pub fn consume_frame(&mut self) -> bool {
        self.frames.consume_ready()
    }

    /// Returns metadata only for bounded logging and host resource decisions.
    #[must_use]
    pub fn state(&self) -> EndpointState {
        EndpointState {
            outgoing_fragments: self.outgoing.queued_fragments(),
            outgoing_bytes: self.outgoing.buffered_bytes(),
            outgoing_acknowledge: self.outgoing.acknowledge(),
            next_outgoing_sequence: self.outgoing.next_sequence(),
            incoming_fragments: self.incoming.stored_fragments(),
            incoming_bytes: self.incoming.buffered_bytes(),
            incoming_acknowledge: self.incoming.acknowledge(),
            frame_ready: self.frames.frame().is_some(),
            failed: self.frames.is_failed(),
        }
    }

    pub(super) fn ensure_active(&self) -> Result<(), EndpointError> {
        if self.frames.is_failed() {
            Err(EndpointError::Failed)
        } else {
            Ok(())
        }
    }
}

impl fmt::Debug for ReliableEndpoint<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReliableEndpoint")
            .field("state", &self.state())
            .finish_non_exhaustive()
    }
}
