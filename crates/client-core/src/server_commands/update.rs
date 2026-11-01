// SPDX-License-Identifier: AGPL-3.0-or-later
/// Bounded-category update metadata, suitable for host aggregation without command text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerCommandUpdate {
    /// The complete validated service message applied by this update.
    pub message_sequence: u32,
    /// Highest retained server-command sequence after commit, initially zero.
    pub newest_command: u32,
    /// Newly retained records; this does not count game/console execution.
    pub new_commands: usize,
    /// Duplicate/older records ignored under native monotonic handling.
    pub ignored_commands: usize,
    /// Sequence numbers skipped by newer records, without fabricating commands.
    pub skipped_sequences: u64,
}
