// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{ClientIdentityChange, ClientIdentitySlot, ClientIdentityState};
use crate::client_data::ClientData;
use std::fmt;

/// Exclusive fully staged identity transaction. Dropping cancels every owner change.
/// The host must validate the entire enclosing message and any sibling application
/// changes before committing. No ACK, command, renderer or network I/O occurs here.
#[must_use = "commit the complete staged transaction, or drop to preserve the owner"]
pub struct PreparedClientIdentities<'owner, 'storage, 'scratch> {
    pub(super) owner: &'owner mut ClientIdentityState<'storage>,
    pub(super) slots: &'scratch [ClientIdentitySlot],
    pub(super) summary: ClientIdentityChange,
}
impl PreparedClientIdentities<'_, '_, '_> {
    /// Returns proposed metadata without applying it or exposing names/tags.
    #[must_use]
    pub fn summary(&self) -> ClientIdentityChange {
        self.summary
    }
    /// Borrows all staged identities for host preflight; no UI or engine state changes.
    pub fn clients(&self) -> impl Iterator<Item = &ClientData> {
        self.slots.iter().filter_map(|slot| slot.0.as_ref())
    }
    /// Copies checked inline records and advances metadata together, without allocation.
    /// Every clone contains fixed wire-field arrays only. Exclusive owner/work borrows
    /// preserve all checked capacities until this infallible commit.
    #[must_use]
    pub fn commit(self) -> ClientIdentityChange {
        self.owner.slots[..self.summary.clients].clone_from_slice(self.slots);
        self.owner.used = self.summary.clients;
        self.owner.generation = self.summary.generation;
        self.owner.sequence = Some(self.summary.sequence);
        self.owner.own_client = Some(self.summary.own_client_index);
        self.summary
    }
}
impl fmt::Debug for PreparedClientIdentities<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedClientIdentities")
            .field("summary", &self.summary)
            .finish_non_exhaustive()
    }
}
