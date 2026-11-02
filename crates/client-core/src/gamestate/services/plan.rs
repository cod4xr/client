// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{GameStateServiceError, GameStateServiceStorage, decode};
use crate::{
    gamestate::{GameStateBudget, GameStatePlan},
    server_payload::DecodedServerPayload,
    service_controls::{ClientServiceBudget, LegacyServicePlan},
};
use std::fmt;

/// Outer control boundary immediately before the native gamestate opcode.
/// Host lifecycle policy must preserve this order around the gamestate baseline;
/// this metadata alone does not admit commands, identities or an epoch transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameStateServicePrefix {
    /// NOP/command/identity records before the gamestate, excluding its opcode.
    pub records: u32,
    /// Leading command records in the complete outer control plan.
    pub commands: usize,
    /// Leading sequenced identity updates, distinct from inner gamestate identities.
    pub client_updates: usize,
}
/// One complete ordinary gamestate and its surrounding controls through outer EOF.
/// The six cooked outputs are inert; no world, connection ACK or engine is applied.
pub struct GameStateServicePlan<'output> {
    pub(super) controls: LegacyServicePlan<'output>,
    pub(super) game_state: GameStatePlan<'output>,
    pub(super) prefix: GameStateServicePrefix,
}
impl<'output> GameStateServicePlan<'output> {
    /// Validates NOP/commands/sequenced identities around exactly one protocol-21
    /// gamestate, including its inner EOF/trailer and the outer logical EOF, before
    /// publishing any final output. Unsupported services are never skipped.
    /// Inner and outer work/capacity budgets remain independent. Native padding
    /// after outer EOF remains uninterpreted.
    ///
    /// # Errors
    /// Profile, malformed controls/gamestate, missing/duplicate gamestate or EOF,
    /// budgets and final capacities preserve all six final output areas. Private
    /// scratch is unspecified on error. No original outgoing ACK is mutated.
    pub fn parse(
        payload: &DecodedServerPayload<'_, '_>,
        services: ClientServiceBudget,
        game_state: GameStateBudget,
        storage: GameStateServiceStorage<'_, 'output>,
    ) -> Result<Self, GameStateServiceError> {
        decode::parse(payload, services, game_state, storage)
    }
    /// Borrows all outer controls in original wire order, including the prefix.
    /// Their record count includes the gamestate opcode but excludes inner records.
    #[must_use]
    pub fn controls(&self) -> &LegacyServicePlan<'output> {
        &self.controls
    }
    /// Borrows the complete constituent, whose server-command baseline stays explicit.
    #[must_use]
    pub fn game_state(&self) -> &GameStatePlan<'output> {
        &self.game_state
    }
    /// Returns counts separating controls before and after the gamestate baseline.
    #[must_use]
    pub fn prefix(&self) -> GameStateServicePrefix {
        self.prefix
    }
}
impl fmt::Debug for GameStateServicePlan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameStateServicePlan")
            .field("profile", &self.controls.profile())
            .field("message_sequence", &self.controls.message_sequence())
            .field("prefix", &self.prefix)
            .field("records", &self.controls.record_count())
            .field("game_state", &self.game_state)
            .finish_non_exhaustive()
    }
}
