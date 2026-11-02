// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded ordinary reliable client-command history, not the auxiliary transport.
//! Caller storage and checked acknowledgements keep private command keys retained.

mod acknowledgement;
mod command;
mod enqueue;
mod error;
mod history;
mod prepared_ack;
#[cfg(test)]
mod sequence_tests;

pub use command::{CheckedClientAcknowledgement, ClientCommand};
pub use error::ClientCommandError;
pub use history::LegacyClientCommands;
pub use prepared_ack::PreparedClientAcknowledgement;

/// Pinned baseline maximum pending ordinary commands; not a custom-engine limit.
pub const MAX_LEGACY_PENDING_COMMANDS: usize = 128;
/// One baseline command slot, including space for its required NUL terminator.
pub const LEGACY_COMMAND_SLOT_BYTES: usize = cod4x_protocol::MAX_COMMAND_BYTES + 1;
/// Maximum baseline caller storage: pending ring plus a retained acknowledged key.
pub const MAX_LEGACY_COMMAND_STORAGE_BYTES: usize =
    (MAX_LEGACY_PENDING_COMMANDS + 1) * LEGACY_COMMAND_SLOT_BYTES;
