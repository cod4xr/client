// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    ClientIdentityBudget, ClientIdentityChange, ClientIdentityError, ClientIdentitySlot,
    ClientIdentityState, PreparedClientIdentities, stage,
};
use crate::client_data::ClientDataUpdate;

impl<'storage> ClientIdentityState<'storage> {
    /// Prepares one complete caller-decoded batch without mutating retained identities.
    /// Apply only an exact wrapping successor. Duplicates, older values and forward
    /// gaps are ignored; none can skip the accepted sequence. Later records in the
    /// same batch compare against the prospective accepted sequence.
    /// The host must first validate the whole enclosing message and bind this owner
    /// to the correct connection. Parsed identities prove neither binding nor identity.
    ///
    /// # Errors
    /// Rejects missing baseline, input/retention budgets and private/final capacities.
    /// Late failures preserve the entire owner, including previously accepted sequence.
    /// Private scratch is unspecified on failure; dropping preparation also cancels.
    ///
    /// A preparation retains exclusive owner access through commit:
    /// ```compile_fail
    /// use cod4xr_client_core::{client_data::ClientDataUpdate, client_identities::{
    ///     ClientIdentityBudget, ClientIdentitySlot, ClientIdentityState,
    /// }};
    /// fn cannot_replace(state: &mut ClientIdentityState<'_>, records: &[ClientDataUpdate],
    ///                   budget: ClientIdentityBudget, scratch: &mut [ClientIdentitySlot]) {
    ///     if let Ok(prepared) = state.prepare_updates(records, budget, scratch) {
    ///         let _ = state.client(0);
    ///         let _ = prepared.commit();
    ///     }
    /// }
    /// ```
    pub fn prepare_updates<'owner, 'scratch>(
        &'owner mut self,
        updates: &[ClientDataUpdate],
        budget: ClientIdentityBudget,
        scratch: &'scratch mut [ClientIdentitySlot],
    ) -> Result<PreparedClientIdentities<'owner, 'storage, 'scratch>, ClientIdentityError> {
        let (Some(sequence), Some(own_client_index)) = (self.sequence, self.own_client) else {
            return Err(ClientIdentityError::MissingBaseline);
        };
        stage::inspect(updates.len(), budget)?;
        if self.used > budget.max_clients {
            return Err(ClientIdentityError::ClientBudget);
        }
        if scratch.len() < self.used {
            return Err(ClientIdentityError::ScratchCapacity);
        }
        scratch[..self.used].clone_from_slice(&self.slots[..self.used]);
        let mut summary = ClientIdentityChange {
            generation: self.generation,
            baseline_replaced: false,
            records: updates.len(),
            accepted_records: 0,
            ignored_records: 0,
            clients: self.used,
            sequence,
            own_client_index,
        };
        for update in updates {
            if update.sequence() != summary.sequence.wrapping_add(1) {
                summary.ignored_records += 1;
                continue;
            }
            stage::upsert(
                scratch,
                &mut summary.clients,
                update.data(),
                budget,
                self.slots.len(),
            )?;
            summary.sequence = update.sequence();
            summary.accepted_records += 1;
        }
        Ok(PreparedClientIdentities {
            owner: self,
            slots: &scratch[..summary.clients],
            summary,
        })
    }
}
