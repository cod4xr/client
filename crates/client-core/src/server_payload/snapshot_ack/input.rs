// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    service_controls::ServiceControlBudget,
    snapshots::{SnapshotInput, services::SnapshotPrediction},
};
use std::fmt;

/// Private decode workspaces, never retained by a cooked snapshot/ACK preparation.
pub struct SnapshotDecodeStorage<'scratch> {
    /// Compatibility-transform workspace; never log its contents.
    pub work: &'scratch mut [u8],
    /// Decompression workspace, still untrusted until complete service validation.
    pub decoded: &'scratch mut [u8],
}
impl fmt::Debug for SnapshotDecodeStorage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotDecodeStorage")
            .field("work_bytes", &self.work.len())
            .field("decoded_bytes", &self.decoded.len())
            .finish_non_exhaustive()
    }
}
/// Explicit world/reference policy and prediction selection for one snapshot carrier.
#[derive(Clone, Copy, Debug)]
pub struct SnapshotPayloadRequest<'source, 'sample> {
    /// Current world/message/context, baselines, coherent prior and sparse budgets.
    pub input: SnapshotInput<'source>,
    /// Complete carrier inspection budget, including NOPs, snapshot and EOF.
    pub services: ServiceControlBudget,
    /// Explicit transmitted/sample/fallback decision, with no automatic selection.
    pub prediction: SnapshotPrediction<'sample>,
}
