// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Duration;

/// Complete challenge-probe limits, never reset by retries or received packets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChallengeBudget {
    /// Absolute lifetime measured from probe creation; must be nonzero.
    pub timeout: Duration,
    /// Delay after each admitted request before another attempt; must be nonzero.
    pub retry_interval: Duration,
    /// Maximum admitted request encodings, including the initial attempt.
    pub max_attempts: u32,
    /// Maximum received datagrams inspected, including wrong endpoints and nonces.
    pub max_datagrams: u32,
}

impl Default for ChallengeBudget {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(2),
            retry_interval: Duration::from_millis(500),
            max_attempts: 3,
            max_datagrams: 64,
        }
    }
}
