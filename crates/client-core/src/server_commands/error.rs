// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed server-command ownership failures without private command/key contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerCommandError {
    /// Metadata slots and complete byte slots have invalid/overflowing geometry.
    Storage,
    /// The requested original command is missing or its slot was reused.
    Missing,
    /// A plan's reviewed message profile differs from the owner.
    Profile,
    /// This validated message is stale or was already applied.
    MessageOrder,
    /// A command cannot fit the baseline stored record or prepared update window.
    CommandCapacity,
    /// A positive wire sequence cannot be represented by host indexing.
    Sequence,
}

impl fmt::Display for ServerCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Storage => "invalid server-command storage geometry",
            Self::Missing => "server command is not retained",
            Self::Profile => "server-command plan profile mismatch",
            Self::MessageOrder => "server-command message is stale or already applied",
            Self::CommandCapacity => "server-command update exceeds its baseline budget",
            Self::Sequence => "server-command sequence exceeds host indexing",
        })
    }
}

impl Error for ServerCommandError {}
