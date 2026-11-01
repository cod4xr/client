// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, net::SocketAddr, time::Duration};

use cod4x_protocol::connection::ChallengeResponse;

use super::ChallengeUsage;

/// Accepted return-path challenge, not server identity or gameplay admission.
/// Only the pending probe can construct this receipt after endpoint/nonce checks.
#[derive(Clone, Copy)]
pub struct ChallengeReceipt {
    pub(super) peer: SocketAddr,
    pub(super) reply: ChallengeResponse,
    pub(super) elapsed: Duration,
    pub(super) usage: ChallengeUsage,
}

impl ChallengeReceipt {
    /// Returns the exact checked peer. No proxy redirection has been accepted.
    #[must_use]
    pub fn peer(&self) -> SocketAddr {
        self.peer
    }

    /// Returns an opaque cookie for a subsequent connect encoder; never log it.
    #[must_use]
    pub fn server_challenge(&self) -> i32 {
        self.reply.server_challenge
    }

    /// Returns elapsed time from the initial probe, including every retry.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Returns final bounded work accounting.
    #[must_use]
    pub fn usage(&self) -> ChallengeUsage {
        self.usage
    }
}

impl fmt::Debug for ChallengeReceipt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChallengeReceipt")
            .field("elapsed", &self.elapsed)
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
