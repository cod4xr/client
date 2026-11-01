// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed failures without peer contents, nonce or return-path cookie disclosure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChallengeFailure {
    /// A timeout, retry interval or work quota was zero.
    InvalidBudget,
    /// The absolute deadline cannot be represented without wrapping.
    TimeOverflow,
    /// Caller-supplied monotonic time moved backwards; state is unchanged.
    TimeWentBackwards,
    /// The absolute deadline has been reached, irrespective of recent traffic.
    Expired,
    /// The complete datagram inspection quota was exhausted without acceptance.
    ReceiveBudget,
    /// Request storage cannot hold the complete wire output; state is unchanged.
    OutputCapacity,
    /// The probe has already accepted, expired or failed.
    Terminal,
}

/// Why one received datagram did not advance a pending challenge probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChallengeRejection {
    /// The complete source endpoint differs from the selected peer.
    WrongPeer,
    /// The command/envelope/integer shape is invalid.
    Malformed,
    /// The named `0 xproto 18` profile did not match; there is no implicit fallback.
    UnsupportedProfile,
    /// The echoed client nonce does not belong to this pending probe.
    WrongNonce,
    /// No challenge request has been admitted yet.
    Unsolicited,
}

impl fmt::Display for ChallengeFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "challenge probe failed: {self:?}")
    }
}

impl Error for ChallengeFailure {}
