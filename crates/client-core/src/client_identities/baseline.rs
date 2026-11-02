// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    ClientIdentityBudget, ClientIdentityChange, ClientIdentityError, ClientIdentitySlot,
    ClientIdentityState, PreparedClientIdentities, stage,
};
use crate::gamestate::ReliableGameStatePlan;

impl<'storage> ClientIdentityState<'storage> {
    /// Prepares a replacement from a complete reliable gamestate wrapper, not a
    /// constituent that could still have an invalid enclosing tail. Last duplicate
    /// client records win, matching native repeated identity assignment.
    /// Old records/sequence/generation remain unchanged until explicit commit.
    /// Staged records retain no plan/frame borrow; the host can commit its prepared
    /// network frame and this table after all fallible resource checks succeed.
    ///
    /// # Errors
    /// Rejects profile/budget/capacity failures and local generation exhaustion.
    /// Private scratch is unspecified on failure; final owner storage is unchanged.
    pub fn prepare_game_state<'owner, 'scratch>(
        &'owner mut self,
        plan: &ReliableGameStatePlan<'_>,
        budget: ClientIdentityBudget,
        scratch: &'scratch mut [ClientIdentitySlot],
    ) -> Result<PreparedClientIdentities<'owner, 'storage, 'scratch>, ClientIdentityError> {
        let game = plan.game_state();
        if game.profile() != self.profile {
            return Err(ClientIdentityError::Profile);
        }
        stage::inspect(game.client_record_count(), budget)?;
        let generation = stage::generation(self.generation)?;
        let mut used = 0;
        for record in game.client_records() {
            stage::upsert(scratch, &mut used, record, budget, self.slots.len())?;
        }
        let summary = ClientIdentityChange {
            generation,
            baseline_replaced: true,
            records: game.client_record_count(),
            accepted_records: game.client_record_count(),
            ignored_records: 0,
            clients: used,
            sequence: game.config_data_sequence(),
            own_client_index: game.client_index(),
        };
        Ok(PreparedClientIdentities {
            owner: self,
            slots: &scratch[..used],
            summary,
        })
    }
}
