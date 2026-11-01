// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Duration;

use super::{ScheduleConfig, ScheduleError};

/// Deterministic timer/cursor state. Windows and clocks belong to the current host session.
#[derive(Debug)]
pub struct ReliableScheduler {
    pub(super) config: ScheduleConfig,
    pub(super) last_now: Duration,
    pub(super) next_any: Duration,
    pub(super) next_data: Duration,
    pub(super) next_ack: Duration,
    pub(super) cursor: u32,
    pub(super) last_ack_base: u32,
    pub(super) ack_cursor: u16,
}

impl ReliableScheduler {
    /// Starts a fresh schedule: data may send immediately; idle ACK waits one interval.
    ///
    /// # Errors
    /// Rejects inconsistent timing and unrepresentable initial ACK deadlines.
    pub fn new(now: Duration, config: ScheduleConfig) -> Result<Self, ScheduleError> {
        let config = config.validate()?;
        let next_ack = now
            .checked_add(config.acknowledgement_interval)
            .ok_or(ScheduleError::ClockOverflow)?;
        Ok(Self {
            config,
            last_now: now,
            next_any: now,
            next_data: now,
            next_ack,
            cursor: 0,
            last_ack_base: 0,
            ack_cursor: 0,
        })
    }

    /// Observes a packet only after complete incoming/outgoing validation and commit.
    /// A force-ACK request advances ACK eligibility but cannot bypass the global interval.
    /// Never call this from an unparsed packet or before whole-packet admission.
    ///
    /// # Errors
    /// Rejects backwards time without changing any timer or cursor.
    pub fn observe_validated_packet(
        &mut self,
        now: Duration,
        force_ack: bool,
    ) -> Result<(), ScheduleError> {
        if now < self.last_now {
            return Err(ScheduleError::ClockRegressed);
        }
        self.last_now = now;
        if force_ack {
            self.next_ack = self.next_ack.min(now);
        }
        Ok(())
    }

    /// Returns the original explicit host policy; peer metadata cannot change it.
    #[must_use]
    pub fn config(&self) -> ScheduleConfig {
        self.config
    }
}
