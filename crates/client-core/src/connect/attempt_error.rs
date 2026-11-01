// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed pending-connect failures with no caller userinfo or server diagnostic bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectFailure {
    /// One phase duration or work quota was zero.
    InvalidBudget,
    /// The absolute phase deadline cannot be represented.
    TimeOverflow,
    /// Host monotonic time moved backwards; no state changes occur.
    TimeWentBackwards,
    /// The original phase deadline has been reached.
    Expired,
    /// The inspection quota ended without a terminal acknowledgement/refusal.
    ReceiveBudget,
    /// Complete request storage does not fit; bytes and state are unchanged.
    OutputCapacity,
    /// The phase already completed, refused, expired or exhausted its quota.
    Terminal,
}

impl fmt::Display for ConnectFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "pending connect failed: {self:?}")
    }
}

impl Error for ConnectFailure {}
