// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::ConnectSummary;

/// Original connection context retained for subsequent profile-specific codecs.
/// This contains a private challenge cookie; Debug deliberately omits its value.
pub struct ChannelContext {
    summary: ConnectSummary,
    challenge: i32,
}

impl ChannelContext {
    pub(crate) fn new(summary: ConnectSummary, challenge: i32) -> Self {
        Self { summary, challenge }
    }

    /// Borrows original protocol/qport and inert acknowledgement metadata.
    #[must_use]
    pub fn summary(&self) -> &ConnectSummary {
        &self.summary
    }

    /// Returns the original challenge for the legacy message obfuscation codec.
    /// This cookie is not authentication; never log or publish it.
    #[must_use]
    pub fn server_challenge(&self) -> i32 {
        self.challenge
    }
}

impl fmt::Debug for ChannelContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChannelContext")
            .field("summary", &self.summary)
            .finish_non_exhaustive()
    }
}
