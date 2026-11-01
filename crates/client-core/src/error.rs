// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed discovery rejection reasons without remote contents or correlation tokens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryError {
    /// Caller output cannot hold the complete request.
    OutputCapacity,
    /// The datagram exceeds this discovery profile or has an invalid envelope.
    InvalidEnvelope,
    /// A different connectionless command or a channel packet was received.
    UnexpectedResponse,
    /// The complete info string has invalid syntax or encoding.
    InvalidInfo,
    /// The query challenge is absent or does not match this request.
    ChallengeMismatch,
    /// Multiple case-insensitive challenge keys are ambiguous and rejected.
    AmbiguousChallenge,
    /// The advertised protocol is missing, empty or outside the unsigned decimal domain.
    InvalidProtocol,
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "discovery rejected: {self:?}")
    }
}

impl Error for DiscoveryError {}
