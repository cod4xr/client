// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller-budgeted normalized server-command retention with deferred atomic updates.
//! Retention may exceed the baseline wire window; no command is executed here.

mod error;
mod history;
mod prepared;
mod slot;
mod update;

pub use error::ServerCommandError;
pub use history::ServerCommandHistory;
pub use prepared::PreparedServerCommands;
pub use slot::ServerCommandSlot;
pub use update::ServerCommandUpdate;

/// One complete baseline big-string slot including its NUL, not an engine string limit.
pub const SERVER_COMMAND_SLOT_BYTES: usize = 8192;

/// Computes byte storage for an operator-selected number of retained slots.
/// There is no native 128-slot retention ceiling; the host owns the memory budget.
///
/// # Errors
/// Rejects zero slots and address-space multiplication overflow without allocating.
pub fn required_server_command_bytes(slots: usize) -> Result<usize, ServerCommandError> {
    slots
        .checked_mul(SERVER_COMMAND_SLOT_BYTES)
        .filter(|&bytes| bytes > 0)
        .ok_or(ServerCommandError::Storage)
}
