// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, time::Duration};

use super::{ReliableScheduler, ScheduleError};

/// Clock/force-ACK observation awaiting whole-packet receive/send validation.
/// Dropping cancels; the exclusive scheduler borrow makes its commit infallible.
#[must_use = "commit only after full packet admission; dropping preserves scheduler state"]
pub struct PreparedPacketObservation<'scheduler> {
    owner: &'scheduler mut ReliableScheduler,
    now: Duration,
    next_ack: Duration,
}

impl ReliableScheduler {
    /// Preflights clock/force-ACK effects without applying them.
    /// Callers must validate all receive and outgoing ACK effects before commit.
    ///
    /// # Errors
    /// Rejects a regressing observation without changing timers or clock state.
    pub fn prepare_packet_observation(
        &mut self,
        now: Duration,
        force_ack: bool,
    ) -> Result<PreparedPacketObservation<'_>, ScheduleError> {
        if now < self.last_now {
            return Err(ScheduleError::ClockRegressed);
        }
        let next_ack = if force_ack {
            self.next_ack.min(now)
        } else {
            self.next_ack
        };
        Ok(PreparedPacketObservation {
            owner: self,
            now,
            next_ack,
        })
    }
}

impl PreparedPacketObservation<'_> {
    /// Applies preflighted clock/force-ACK metadata after complete packet admission.
    pub fn commit(self) {
        self.owner.last_now = self.now;
        self.owner.next_ack = self.next_ack;
    }
}

impl fmt::Debug for PreparedPacketObservation<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedPacketObservation")
            .field("at", &self.now)
            .field("next_acknowledgement", &self.next_ack)
            .finish_non_exhaustive()
    }
}
