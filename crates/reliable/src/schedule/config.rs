// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Duration;

use super::ScheduleError;

/// Explicit host timing policy, independent of peer flags/hints and queue capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScheduleConfig {
    /// Minimum interval between all successful datagram admissions, even forced ACKs.
    pub minimum_interval: Duration,
    /// Interval between data/retransmission admissions.
    pub data_interval: Duration,
    /// Periodic ACK interval, refreshed by either a data or ACK-only admission.
    pub acknowledgement_interval: Duration,
}

impl Default for ScheduleConfig {
    /// Uses native default data/ACK intervals and a host policy cap of 50 datagrams/sec.
    fn default() -> Self {
        Self {
            minimum_interval: Duration::from_millis(20),
            data_interval: Duration::from_millis(250),
            acknowledgement_interval: Duration::from_millis(350),
        }
    }
}

impl ScheduleConfig {
    pub(super) fn validate(self) -> Result<Self, ScheduleError> {
        if self.minimum_interval.is_zero()
            || self.data_interval.is_zero()
            || self.acknowledgement_interval.is_zero()
            || self.minimum_interval > self.data_interval
            || self.minimum_interval > self.acknowledgement_interval
        {
            return Err(ScheduleError::Configuration);
        }
        Ok(self)
    }
}
