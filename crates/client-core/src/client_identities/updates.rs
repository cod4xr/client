// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    ClientIdentityBudget, ClientIdentityError, ClientIdentitySlot, ClientIdentityState,
    PreparedClientIdentities,
};
use crate::client_data::ClientDataUpdate;
use crate::service_controls::LegacyServicePlan;

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
        self.prepare_batch(updates.len(), updates.iter(), budget, scratch)
    }
    /// Stages updates only from a complete validated service plan of the same profile.
    /// The whole stream, including every command/identity and logical EOF, has already
    /// validated. Commands and outgoing acknowledgement remain separate sibling
    /// transactions for host preflight; nothing executes or commits here.
    ///
    /// # Errors
    /// Rejects a different profile and every error from [`Self::prepare_updates`].
    /// Late failures preserve the entire owner; private scratch is unspecified.
    pub fn prepare_service_plan<'owner, 'scratch>(
        &'owner mut self,
        plan: &LegacyServicePlan<'_>,
        budget: ClientIdentityBudget,
        scratch: &'scratch mut [ClientIdentitySlot],
    ) -> Result<PreparedClientIdentities<'owner, 'storage, 'scratch>, ClientIdentityError> {
        if plan.profile() != self.profile {
            return Err(ClientIdentityError::Profile);
        }
        self.prepare_batch(
            plan.client_update_count(),
            plan.client_updates(),
            budget,
            scratch,
        )
    }
}
