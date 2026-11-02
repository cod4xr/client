// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::{ClientSender, NetworkError, OutboundMessage, SendUsage};
use cod4xr_client_core::control::ControlMessagePlan;

/// Narrow ordinary-message borrow of one reliable connection's serialized sender.
/// It cannot emit external auxiliary windows or bypass the owner's clock/pacing state.
pub struct OrdinaryConnectionSender<'connection> {
    inner: ClientSender<'connection>,
}

impl<'connection> OrdinaryConnectionSender<'connection> {
    pub(super) fn new(inner: ClientSender<'connection>) -> Self {
        Self { inner }
    }

    /// Starts an ordinary control frame after the original protocol/cookie context check.
    ///
    /// # Errors
    /// Rejects another connection's plan and shared channel size/sequence failures.
    pub fn begin_control<'sender, 'message>(
        &'sender mut self,
        plan: &'message ControlMessagePlan,
    ) -> Result<OutboundMessage<'sender, 'message>, NetworkError> {
        self.inner.begin_control(plan)
    }

    /// Starts one explicitly encoded ordinary body, independently of auxiliary sequencing.
    ///
    /// # Errors
    /// Rejects shared ordinary body size and exhausted sequence space.
    pub fn begin<'sender, 'message>(
        &'sender mut self,
        body: &'message [u8],
    ) -> Result<OutboundMessage<'sender, 'message>, NetworkError> {
        self.inner.begin(body)
    }

    /// Returns the next ordinary sequence, without reserving it.
    #[must_use]
    pub fn next_sequence(&self) -> u32 {
        self.inner.next_sequence()
    }

    /// Returns copied I/O totals, including the connection's auxiliary subset counters.
    #[must_use]
    pub fn usage(&self) -> SendUsage {
        self.inner.usage()
    }
}

impl fmt::Debug for OrdinaryConnectionSender<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OrdinaryConnectionSender")
            .field("usage", &self.usage())
            .finish_non_exhaustive()
    }
}
