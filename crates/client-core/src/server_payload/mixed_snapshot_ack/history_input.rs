// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    entities::EntityDecodeContext,
    service_controls::ClientServiceBudget,
    snapshots::{
        SnapshotBaselines, SnapshotBudget, SnapshotMessage, history::SnapshotHistory,
        services::SnapshotPrediction,
    },
};

/// Explicit current world and retained owner for actual-header reference selection.
/// No prior is supplied separately; only the exact decoded wire distance selects it.
#[derive(Clone, Copy, Debug)]
pub struct MixedSnapshotHistoryRequest<'source, 'slots, 'storage, 'sample> {
    /// Current coordinate context; it must match the original decoded profile.
    pub context: EntityDecodeContext,
    /// Actual host world/message and preceding accepted command high-water mark.
    pub message: SnapshotMessage,
    /// Current world/profile initial entity baselines.
    pub baselines: SnapshotBaselines<'source>,
    /// Independent sparse source/work/output bounds.
    pub budget: SnapshotBudget,
    /// Independent total service and identity inspection bounds.
    pub services: ClientServiceBudget,
    /// Mandatory explicit prediction choice.
    pub prediction: SnapshotPrediction<'sample>,
    /// Original current-world retained snapshots; no history borrow survives success.
    pub history: &'source SnapshotHistory<'slots, 'storage>,
}
