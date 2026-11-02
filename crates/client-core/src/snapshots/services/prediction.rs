// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{PreparedSnapshotServices, SnapshotServiceError, SnapshotServicePlan};
use crate::player_state::history::PredictionHistorySource;

impl<'storage> PreparedSnapshotServices<'_, '_, '_, 'storage> {
    /// Resolves exact retained prediction after this complete carrier passed EOF.
    /// Transmitted mode still checks the selected source's world/profile binding.
    ///
    /// # Errors
    /// Binding or missing exact samples cancel all reader/output publication. No
    /// prior fallback is inferred; the result retains no history source borrow.
    pub fn commit_from_history(
        self,
        history: PredictionHistorySource<'_>,
    ) -> Result<SnapshotServicePlan<'storage>, SnapshotServiceError> {
        let snapshot = self
            .snapshot
            .commit_from_history(history)
            .map_err(SnapshotServiceError::Snapshot)?;
        Ok(SnapshotServicePlan {
            snapshot,
            record_count: self.record_count,
            consumed_bytes: self.consumed_bytes,
        })
    }
}
