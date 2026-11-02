// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{PreparedSnapshotRetention, SnapshotHistoryError, SnapshotHistorySlot};
use crate::{
    BaselineProtocol,
    snapshots::{SnapshotMessage, SnapshotPlan},
};
use std::fmt;

/// Exact-sequence, epoch-bound coherent history with host-selected slot capacities.
/// Commits replace slots in insertion order; sequence gaps do not consume slots.
/// References borrow the owner, preventing reset or eviction while in use.
///
/// ```compile_fail,E0502
/// use cod4xr_client_core::{BaselineProtocol, snapshots::history::SnapshotHistory};
/// fn cannot_reset(history: &mut SnapshotHistory<'_, '_>) {
///     let source = history.source(1).unwrap();
///     history.reset(2, BaselineProtocol::Cod4x21).unwrap();
///     let _ = source.message();
/// }
/// ```
pub struct SnapshotHistory<'slots, 'storage> {
    pub(super) slots: &'slots mut [SnapshotHistorySlot<'storage>],
    pub(super) epoch: u64,
    pub(super) profile: BaselineProtocol,
    pub(super) next: usize,
    pub(super) used: usize,
    pub(super) newest: Option<u32>,
}
impl<'slots, 'storage> SnapshotHistory<'slots, 'storage> {
    /// Starts a fresh explicit world/profile and invalidates old slot metadata.
    /// Raw backing cells and unused tails remain unchanged. Epoch uniqueness across
    /// newly constructed owners is the host's responsibility, not authentication.
    ///
    /// # Errors
    /// An empty slot slice rejects without modifying any slot.
    pub fn new(
        epoch: u64,
        profile: BaselineProtocol,
        slots: &'slots mut [SnapshotHistorySlot<'storage>],
    ) -> Result<Self, SnapshotHistoryError> {
        if slots.is_empty() {
            return Err(SnapshotHistoryError::Storage);
        }
        for slot in &mut *slots {
            slot.retained = None;
        }
        Ok(Self {
            slots,
            epoch,
            profile,
            next: 0,
            used: 0,
            newest: None,
        })
    }
    /// Invalidates all references for a new map/connection/gamestate epoch.
    /// No raw cell clearing, allocation or engine/session effect occurs.
    ///
    /// # Errors
    /// Equal or older epochs reject without modifying metadata or backing cells.
    pub fn reset(
        &mut self,
        epoch: u64,
        profile: BaselineProtocol,
    ) -> Result<(), SnapshotHistoryError> {
        if epoch <= self.epoch {
            return Err(SnapshotHistoryError::EpochOrder);
        }
        for slot in &mut *self.slots {
            slot.retained = None;
        }
        self.epoch = epoch;
        self.profile = profile;
        self.next = 0;
        self.used = 0;
        self.newest = None;
        Ok(())
    }
    /// Returns the local world epoch, without peer identity claims.
    #[must_use]
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    /// Returns the explicit retained profile.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns caller-funded snapshot slots, independent of the native 32-entry ring.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
    /// Returns currently committed distinct snapshots.
    #[must_use]
    pub fn len(&self) -> usize {
        self.used
    }
    /// Reports whether no snapshot has been committed in this epoch.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.used == 0
    }
    /// Returns the last committed containing-message sequence, not transport receipt.
    #[must_use]
    pub fn newest_sequence(&self) -> Option<u32> {
        self.newest
    }
    pub(super) fn check(
        &self,
        message: SnapshotMessage,
        profile: BaselineProtocol,
    ) -> Result<(), SnapshotHistoryError> {
        if message.epoch() != self.epoch {
            return Err(SnapshotHistoryError::Epoch);
        }
        if profile != self.profile {
            return Err(SnapshotHistoryError::Profile);
        }
        if self
            .newest
            .is_some_and(|sequence| message.sequence() <= sequence)
        {
            return Err(SnapshotHistoryError::MessageOrder);
        }
        Ok(())
    }
    /// Checks next-slot storage and binding before changing history or snapshot data.
    /// Commit this child only after every sibling resource preflight has succeeded.
    /// Dropping the exclusive prepared lease leaves all history cells unchanged.
    ///
    /// # Errors
    /// Rejects mismatched epoch/profile, stale message sequence and insufficient
    /// entity/client capacity in the next insertion-order slot. No eviction occurs.
    pub fn prepare<'owner, 'plan, 'output>(
        &'owner mut self,
        plan: &'plan SnapshotPlan<'output>,
    ) -> Result<
        PreparedSnapshotRetention<'owner, 'slots, 'storage, 'plan, 'output>,
        SnapshotHistoryError,
    > {
        self.check(plan.message(), plan.header().profile())?;
        let target = &self.slots[self.next];
        if plan.entities().retained_slots().len() > target.entities.len() {
            return Err(SnapshotHistoryError::EntityCapacity);
        }
        if plan.clients().retained_slots().len() > target.clients.len() {
            return Err(SnapshotHistoryError::ClientCapacity);
        }
        Ok(PreparedSnapshotRetention { owner: self, plan })
    }
}
impl fmt::Debug for SnapshotHistory<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotHistory")
            .field("profile", &self.profile)
            .field("capacity", &self.slots.len())
            .field("retained", &self.used)
            .finish_non_exhaustive()
    }
}
