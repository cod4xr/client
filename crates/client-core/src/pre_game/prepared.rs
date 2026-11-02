// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{PreGameError, PreGameEvent};
use crate::{
    server_commands::{PreparedServerCommands, ServerCommandUpdate},
    service_controls::MAX_BASELINE_SERVER_COMMANDS,
};
use std::fmt;

/// Independent host work/publication budgets for newly admitted commands only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreGameBudget {
    /// Sum of complete normalized bytes in every new command, including unknown ones.
    pub max_command_bytes: usize,
    /// Maximum recognized notices. Unknown commands consume bytes but no event slot.
    pub max_events: usize,
}

struct Events<'plan> {
    slots: [Option<PreGameEvent<'plan>>; MAX_BASELINE_SERVER_COMMANDS],
    count: usize,
}
impl Events<'_> {
    fn iter(&self) -> impl Iterator<Item = PreGameEvent<'_>> {
        self.slots[..self.count].iter().filter_map(|event| *event)
    }
}

/// Exclusive command-history preparation plus fully checked inert pre-game notices.
/// Dropping cancels both. No session, console or network state changes here.
/// Exclusive history access persists through typed preflight:
/// ```compile_fail
/// use cod4xr_client_core::{pre_game::PreGameBudget,
///     server_commands::ServerCommandHistory, service_controls::LegacyServicePlan};
/// fn cannot_replace(history: &mut ServerCommandHistory<'_>, plan: &LegacyServicePlan<'_>,
///                   budget: PreGameBudget) {
///     if let Ok(commands) = history.prepare(plan) {
///         if let Ok(typed) = commands.with_pre_game(budget) {
///             let _ = history.newest_sequence();
///             let _ = typed.commit();
///         }
///     }
/// }
/// ```
#[must_use = "preflight sibling resources before commit, or drop to cancel"]
pub struct PreparedPreGameCommands<'store, 'plan, 'buffers> {
    commands: PreparedServerCommands<'store, 'plan, 'buffers>,
    events: Events<'plan>,
}

/// Notices returned only after the corresponding command history has committed.
/// The host must still bind its original connection, apply phase/content policy
/// and keep all returned text out of logs or raw command execution.
pub struct CommittedPreGameCommands<'plan> {
    summary: ServerCommandUpdate,
    events: Events<'plan>,
}

impl<'store, 'plan, 'buffers> PreparedServerCommands<'store, 'plan, 'buffers> {
    /// Preflights typed notices from precisely the new records selected by history.
    /// Duplicate/older commands never emit notices. All input records were already
    /// validated through service EOF; the lexer borrows only normalized plan bytes.
    ///
    /// # Errors
    /// Rejects byte/event budgets without committing history. The consumed preparation
    /// drops on error, releasing its exclusive lease and leaving the owner unchanged.
    pub fn with_pre_game(
        self,
        budget: PreGameBudget,
    ) -> Result<PreparedPreGameCommands<'store, 'plan, 'buffers>, PreGameError> {
        let mut events = Events {
            slots: [None; MAX_BASELINE_SERVER_COMMANDS],
            count: 0,
        };
        let mut bytes = 0_usize;
        for command in self.new_commands() {
            bytes = bytes
                .checked_add(command.as_bytes().len())
                .filter(|&total| total <= budget.max_command_bytes)
                .ok_or(PreGameError::CommandByteBudget)?;
            if let Some(event) = PreGameEvent::inspect(command) {
                if events.count >= budget.max_events {
                    return Err(PreGameError::EventBudget);
                }
                events.slots[events.count] = Some(event);
                events.count += 1;
            }
        }
        Ok(PreparedPreGameCommands {
            commands: self,
            events,
        })
    }
}
impl<'plan> PreparedPreGameCommands<'_, 'plan, '_> {
    /// Returns proposed history metadata without committing it.
    #[must_use]
    pub fn summary(&self) -> ServerCommandUpdate {
        self.commands.summary()
    }
    /// Borrows inert staged notices for sibling resource/phase preflight only.
    pub fn events(&self) -> impl Iterator<Item = PreGameEvent<'_>> {
        self.events.iter()
    }
    /// Commits command retention and returns its typed notices without executing them.
    /// All sibling checks and the checked outgoing ACK must have succeeded first.
    #[must_use]
    pub fn commit(self) -> CommittedPreGameCommands<'plan> {
        CommittedPreGameCommands {
            summary: self.commands.commit(),
            events: self.events,
        }
    }
}
impl CommittedPreGameCommands<'_> {
    /// Returns committed command metadata for counters and the next control header.
    #[must_use]
    pub fn summary(&self) -> ServerCommandUpdate {
        self.summary
    }
    /// Borrows committed inert notices, in original new-command order.
    pub fn events(&self) -> impl Iterator<Item = PreGameEvent<'_>> {
        self.events.iter()
    }
}
impl fmt::Debug for PreparedPreGameCommands<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedPreGameCommands")
            .field("summary", &self.summary())
            .field("events", &self.events.count)
            .finish_non_exhaustive()
    }
}
impl fmt::Debug for CommittedPreGameCommands<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommittedPreGameCommands")
            .field("summary", &self.summary)
            .field("events", &self.events.count)
            .finish_non_exhaustive()
    }
}
