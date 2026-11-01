// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed framing errors without body bytes or private command data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameError {
    /// The complete four-byte length prefix is missing.
    Truncated,
    /// A length is negative or not representable in the signed native/host domain.
    Length,
    /// Complete-frame input differs from the exact declared body size.
    LengthMismatch,
    /// The peer-declared body exceeds the explicit caller-provided storage budget.
    BodyCapacity,
    /// Caller output cannot hold the entire encoded frame.
    OutputCapacity,
    /// A previous invalid length poisoned this stream; reconnect instead of resyncing.
    Failed,
}

impl fmt::Display for FrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Truncated => "incomplete reliable application length prefix",
            Self::Length => "invalid reliable application body length",
            Self::LengthMismatch => "reliable application length differs from complete frame bytes",
            Self::BodyCapacity => "reliable application body exceeds host storage budget",
            Self::OutputCapacity => "reliable application frame output capacity insufficient",
            Self::Failed => "reliable application stream failed; reconnect required",
        })
    }
}

impl Error for FrameError {}
