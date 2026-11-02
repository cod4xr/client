// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    ClientIdentityBudget, ClientIdentityChange, ClientIdentityError, ClientIdentitySlot,
    ClientIdentityState, PreparedClientIdentities, stage,
};
use crate::client_data::ClientDataUpdate;

impl<'storage> ClientIdentityState<'storage> {
    // Only closed slice/validated-plan adapters call this private iterator path.
    // Public callers cannot inject iterator callbacks or misstate the input count.
    pub(super) fn prepare_batch<'owner, 'scratch, 'updates>(
        &'owner mut self,
        records: usize,
        updates: impl Iterator<Item = &'updates ClientDataUpdate>,
        budget: ClientIdentityBudget,
        scratch: &'scratch mut [ClientIdentitySlot],
    ) -> Result<PreparedClientIdentities<'owner, 'storage, 'scratch>, ClientIdentityError> {
        let (Some(sequence), Some(own_client_index)) = (self.sequence, self.own_client) else {
            return Err(ClientIdentityError::MissingBaseline);
        };
        stage::inspect(records, budget)?;
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
            records,
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
