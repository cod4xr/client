// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    service_controls::ClientServiceBudget,
    snapshots::{SnapshotInput, services::SnapshotPrediction},
};

/// Explicit world/reference/control limits and prediction selection for one carrier.
#[derive(Clone, Copy, Debug)]
pub struct MixedSnapshotRequest<'source, 'sample> {
    /// Current world/message/context and independent sparse source/storage budgets.
    pub input: SnapshotInput<'source>,
    /// All service/identity inspection bounds, including the snapshot and EOF.
    pub services: ClientServiceBudget,
    /// Mandatory transmitted/sample/native-fallback decision.
    pub prediction: SnapshotPrediction<'sample>,
}
