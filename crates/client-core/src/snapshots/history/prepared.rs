// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{SnapshotHistory, slot::Retained};
use crate::snapshots::SnapshotPlan;
use std::fmt;

/// Bounded commitment evidence without raw state words, time or world identifiers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotRetention {
    /// Containing-message sequence newly retained.
    pub sequence: u32,
    /// Exact old sequence evicted from the selected slot, if occupied.
    pub evicted: Option<u32>,
    /// Current committed snapshot count.
    pub retained: usize,
}
/// Exclusive original history owner and immutable complete cooked snapshot.
/// Dropping cancels retention without altering any metadata, cells or counters.
///
/// ```compile_fail,E0499
/// use cod4xr_client_core::snapshots::{SnapshotPlan, history::SnapshotHistory};
/// fn cannot_replace(history: &mut SnapshotHistory<'_, '_>, plan: &SnapshotPlan<'_>) {
///     let prepared = history.prepare(plan).unwrap();
///     let _ = history.prepare(plan);
///     let _ = prepared.commit();
/// }
/// ```
#[must_use = "Commit only after all sibling preflight, or drop to cancel retention."]
pub struct PreparedSnapshotRetention<'owner, 'slots, 'storage, 'plan, 'output> {
    pub(super) owner: &'owner mut SnapshotHistory<'slots, 'storage>,
    pub(super) plan: &'plan SnapshotPlan<'output>,
}
impl PreparedSnapshotRetention<'_, '_, '_, '_, '_> {
    /// Copies already checked complete state into the selected host-funded slot.
    /// No parsing, allocation, callback, I/O or fallible check runs during commit.
    /// All source/target capacities and bindings remain held by exclusive borrows.
    #[must_use]
    pub fn commit(self) -> SnapshotRetention {
        let owner = self.owner;
        let target = &mut owner.slots[owner.next];
        let evicted = target
            .retained
            .as_ref()
            .map(|state| state.message.sequence());
        let entities = self.plan.entities().retained_slots();
        let clients = self.plan.clients().retained_slots();
        target.entities[..entities.len()].clone_from_slice(entities);
        target.clients[..clients.len()].clone_from_slice(clients);
        target.retained = Some(Retained {
            header: self.plan.header(),
            message: self.plan.message(),
            player: self.plan.player().clone(),
            entities: entities.len(),
            clients: clients.len(),
        });
        owner.newest = Some(self.plan.message().sequence());
        owner.used += usize::from(owner.used < owner.slots.len());
        owner.next = if owner.next == owner.slots.len() - 1 {
            0
        } else {
            owner.next + 1
        };
        SnapshotRetention {
            sequence: self.plan.message().sequence(),
            evicted,
            retained: owner.used,
        }
    }
}
impl fmt::Debug for PreparedSnapshotRetention<'_, '_, '_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedSnapshotRetention")
            .field("history", &self.owner)
            .finish_non_exhaustive()
    }
}
