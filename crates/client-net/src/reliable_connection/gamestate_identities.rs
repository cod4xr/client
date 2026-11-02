// SPDX-License-Identifier: AGPL-3.0-or-later
use super::PreparedGameStateFrame;
use cod4xr_client_core::{
    client_data::ClientData,
    client_identities::{
        ClientIdentityBudget, ClientIdentityChange, ClientIdentityError, ClientIdentitySlot,
        ClientIdentityState, PreparedClientIdentities,
    },
    gamestate::ReliableGameStatePlan,
};
use std::fmt;

/// One joint transaction retaining the same frame and staged identity owner.
/// Dropping cancels both release and identity changes. The host still owns peer
/// binding, other session/content preflight and engine application.
#[must_use = "commit the joint frame/identity transaction, or drop to cancel both"]
pub struct PreparedIdentityGameStateFrame<'owner, 'network, 'output, 'identity, 'storage, 'scratch>
{
    frame: PreparedGameStateFrame<'owner, 'network, 'output>,
    identities: PreparedClientIdentities<'identity, 'storage, 'scratch>,
}
impl<'owner, 'network, 'output> PreparedGameStateFrame<'owner, 'network, 'output> {
    /// Stages identities from exactly this retained complete gamestate and combines
    /// both exclusive leases. No owner changes, frame release, ACK or I/O occurs.
    /// Staging clones fixed inline records, without retaining an extra source borrow.
    ///
    /// # Errors
    /// Rejects identity profile/budget/capacity/generation failures. The consumed
    /// frame preparation drops on error, leaving the same network frame ready and
    /// all identity state unchanged. Private staging is unspecified on failure.
    pub fn with_client_identities<'identity, 'storage, 'scratch>(
        self,
        state: &'identity mut ClientIdentityState<'storage>,
        budget: ClientIdentityBudget,
        scratch: &'scratch mut [ClientIdentitySlot],
    ) -> Result<
        PreparedIdentityGameStateFrame<'owner, 'network, 'output, 'identity, 'storage, 'scratch>,
        ClientIdentityError,
    > {
        let identities = state.prepare_game_state(self.plan(), budget, scratch)?;
        Ok(PreparedIdentityGameStateFrame {
            frame: self,
            identities,
        })
    }
}
impl<'output> PreparedIdentityGameStateFrame<'_, '_, 'output, '_, '_, '_> {
    /// Borrows the whole cooked gamestate for further bounded session/content preflight.
    #[must_use]
    pub fn plan(&self) -> &ReliableGameStatePlan<'output> {
        self.frame.plan()
    }
    /// Returns proposed identity counters/sequence without applying them.
    #[must_use]
    pub fn identity_change(&self) -> ClientIdentityChange {
        self.identities.summary()
    }
    /// Borrows staged inert identities without UI, logging or authentication effects.
    pub fn clients(&self) -> impl Iterator<Item = &ClientData> {
        self.identities.clients()
    }
    /// Releases exactly this frame and commits the staged identity state together.
    /// Both operations are fully preflighted and infallible, with no callbacks or
    /// I/O between them. Byte receipt ACKs remain distinct from application commit.
    /// The returned cooked plan retains no network, staging or identity-owner borrow.
    #[must_use]
    pub fn commit(self) -> (ReliableGameStatePlan<'output>, ClientIdentityChange) {
        (self.frame.commit(), self.identities.commit())
    }
}
impl fmt::Debug for PreparedIdentityGameStateFrame<'_, '_, '_, '_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedIdentityGameStateFrame")
            .field("frame", &self.frame)
            .field("identities", &self.identities)
            .finish_non_exhaustive()
    }
}
