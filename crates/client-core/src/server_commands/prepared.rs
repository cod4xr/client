// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use super::{
    SERVER_COMMAND_SLOT_BYTES, ServerCommandError, ServerCommandHistory, ServerCommandSlot,
    ServerCommandUpdate,
};
use crate::service_controls::{LegacyServicePlan, MAX_BASELINE_SERVER_COMMANDS};

#[derive(Clone, Copy, Default)]
struct PreparedCommand<'plan> {
    index: usize,
    sequence: u32,
    bytes: &'plan [u8],
}

/// Exclusively borrowed, fully preflighted incoming update. Dropping cancels it.
/// The host can validate/commit its separate outgoing acknowledgement before this
/// infallible commit; no retained command changes during preparation.
#[must_use = "commit the prepared update to retain commands; dropping cancels without mutation"]
pub struct PreparedServerCommands<'store, 'plan, 'buffers> {
    owner: &'store mut ServerCommandHistory<'buffers>,
    commands: [PreparedCommand<'plan>; MAX_BASELINE_SERVER_COMMANDS],
    summary: ServerCommandUpdate,
}

impl<'buffers> ServerCommandHistory<'buffers> {
    /// Preflights a whole validated plan without changing retained bytes or counters.
    /// Newer records advance a prospective high-water mark; duplicates/older records
    /// are ignored, including records appearing after a newer one in the same plan.
    ///
    /// # Errors
    /// Rejects profile/order mismatches and any unrepresentable routing/length budget.
    pub fn prepare<'store, 'plan>(
        &'store mut self,
        plan: &'plan LegacyServicePlan<'_>,
    ) -> Result<PreparedServerCommands<'store, 'plan, 'buffers>, ServerCommandError> {
        if plan.profile() != self.profile {
            return Err(ServerCommandError::Profile);
        }
        if plan.message_sequence() <= self.last_message {
            return Err(ServerCommandError::MessageOrder);
        }
        let mut commands = [PreparedCommand::default(); MAX_BASELINE_SERVER_COMMANDS];
        let mut summary = ServerCommandUpdate {
            message_sequence: plan.message_sequence(),
            newest_command: self.newest,
            new_commands: 0,
            ignored_commands: 0,
            skipped_sequences: 0,
        };
        for command in plan.commands() {
            let sequence = command.sequence();
            if sequence <= summary.newest_command {
                summary.ignored_commands += 1;
                continue;
            }
            if command.as_bytes().len() >= SERVER_COMMAND_SLOT_BYTES
                || summary.new_commands == commands.len()
            {
                return Err(ServerCommandError::CommandCapacity);
            }
            let index = usize::try_from(sequence).map_err(|_| ServerCommandError::Sequence)?
                % self.slots.len();
            commands[summary.new_commands] = PreparedCommand {
                index,
                sequence,
                bytes: command.as_bytes(),
            };
            summary.new_commands += 1;
            summary.skipped_sequences += u64::from(sequence - summary.newest_command - 1);
            summary.newest_command = sequence;
        }
        Ok(PreparedServerCommands {
            owner: self,
            commands,
            summary,
        })
    }
}

impl PreparedServerCommands<'_, '_, '_> {
    /// Returns proposed counters without applying them or exposing private data.
    #[must_use]
    pub fn summary(&self) -> ServerCommandUpdate {
        self.summary
    }

    /// Applies already checked writes and counters without allocation or fallible I/O.
    /// Exclusive owner/data borrows preserve every preflighted bound until commit.
    #[must_use]
    pub fn commit(self) -> ServerCommandUpdate {
        let owner = self.owner;
        for command in &self.commands[..self.summary.new_commands] {
            let start = command.index * SERVER_COMMAND_SLOT_BYTES;
            let target = &mut owner.bytes[start..start + SERVER_COMMAND_SLOT_BYTES];
            target[..command.bytes.len()].copy_from_slice(command.bytes);
            target[command.bytes.len()] = 0;
            owner.slots[command.index] = ServerCommandSlot {
                sequence: command.sequence,
                length: command.bytes.len(),
            };
        }
        owner.newest = self.summary.newest_command;
        owner.last_message = self.summary.message_sequence;
        self.summary
    }
}

impl fmt::Debug for PreparedServerCommands<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedServerCommands")
            .field("summary", &self.summary)
            .finish_non_exhaustive()
    }
}
