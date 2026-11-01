// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed reply-shape failures without untrusted mod labels or server messages.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectReplyError {
    /// The packet is not a bounded connectionless envelope.
    Envelope,
    /// The command header or field count is invalid for this profile.
    Syntax,
    /// The command is outside connectResponse/error/print handling.
    Unsupported,
    /// Opaque text exceeds the named legacy destination's complete byte budget.
    Capacity,
    /// Extra bytes follow an accepted command or embedded string terminator.
    TrailingData,
}

impl fmt::Display for ConnectReplyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "connect reply rejected: {self:?}")
    }
}

impl Error for ConnectReplyError {}
