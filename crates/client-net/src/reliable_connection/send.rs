// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Instant;

use super::{ConnectionSend, ReliableConnection};
use crate::NetworkError;
use cod4xr_reliable::schedule::SchedulePoll;

impl ReliableConnection<'_> {
    /// Uses actual elapsed host time to send at most one due datagram.
    /// A wait result is capped at the caller's absolute deadline. There is no catch-up loop.
    ///
    /// # Errors
    /// Rejects terminal framing/clock/window errors and propagates codec/deadline/socket
    /// failures. Failed I/O admits neither a fragment nor its scheduling lease.
    pub fn send_due(&mut self, deadline: Instant) -> Result<ConnectionSend, NetworkError> {
        self.ensure_active()?;
        let now = self.started.elapsed();
        match self.endpoint.poll(now).map_err(NetworkError::Endpoint)? {
            SchedulePoll::WaitUntil(at) => {
                let wake = self
                    .started
                    .checked_add(at)
                    .ok_or(NetworkError::ClockOverflow)?;
                Ok(ConnectionSend::WaitUntil(wake.min(deadline)))
            }
            SchedulePoll::Send(plan) => {
                let sent = self
                    .channel
                    .split()
                    .sender
                    .send_scheduled_reliable(plan, deadline)?;
                Ok(ConnectionSend::Sent(sent))
            }
        }
    }
}
