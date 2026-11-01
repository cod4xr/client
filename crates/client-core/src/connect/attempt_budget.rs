// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Duration;

/// One pending connect phase's complete resource limits, never reset by retries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectBudget {
    /// Absolute phase lifetime from construction; must be nonzero.
    pub timeout: Duration,
    /// Delay between admitted request encodings; must be nonzero.
    pub retry_interval: Duration,
    /// Maximum admitted encodings, including the initial request.
    pub max_attempts: u32,
    /// Maximum inspected datagrams, including notices and rejected traffic.
    pub max_datagrams: u32,
}

impl Default for ConnectBudget {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(3),
            retry_interval: Duration::from_millis(500),
            max_attempts: 3,
            max_datagrams: 64,
        }
    }
}
