// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{SnapshotBaselines, SnapshotMessage, SnapshotSource};
use crate::{
    entities::EntityDecodeContext,
    snapshot_clients::{ClientStreamBudget, SnapshotClientSlot},
    snapshot_entities::{EntityStreamBudget, SnapshotEntitySlot},
};
use std::fmt;

/// Independent host work/retention limits for the two sparse substreams.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotBudget {
    /// Entity source, record and final-retention budgets.
    pub entities: EntityStreamBudget,
    /// Client-state source, record and final-retention budgets.
    pub clients: ClientStreamBudget,
}
/// Explicit codec context and reference bindings, independent of discovery/global state.
#[derive(Clone, Copy, Debug)]
pub struct SnapshotInput<'source> {
    /// Current explicit coordinate/profile context.
    pub context: EntityDecodeContext,
    /// Caller-owned world and containing-message association.
    pub message: SnapshotMessage,
    /// Exact cooked prior for a nonzero distance; absent for a full snapshot.
    pub previous: Option<SnapshotSource<'source>>,
    /// Initial baselines from the current world epoch/profile.
    pub baselines: SnapshotBaselines<'source>,
    /// Independent source/work/retention policy.
    pub budget: SnapshotBudget,
}
/// Final caller-funded cells, preserved in full until whole-snapshot commitment.
pub struct SnapshotOutput<'storage> {
    /// Complete merged entity output; unused tails remain unchanged.
    pub entities: &'storage mut [SnapshotEntitySlot],
    /// Complete merged client-state output; unused tails remain unchanged.
    pub clients: &'storage mut [SnapshotClientSlot],
}
impl fmt::Debug for SnapshotOutput<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotOutput")
            .field("entity_capacity", &self.entities.len())
            .field("client_capacity", &self.clients.len())
            .finish_non_exhaustive()
    }
}
