// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{PreparedSnapshot, SnapshotError, SnapshotPlan};
use crate::player_state::history::PredictionHistorySource;

impl<'storage> PreparedSnapshot<'_, '_, '_, 'storage> {
    /// Commits using exact current-world retained prediction when the wire requests
    /// it, or transmitted words otherwise. No missing/older-sample fallback occurs.
    /// The actual decoded command time selects the sample after complete body
    /// validation; no lookup time is supplied separately by the caller.
    ///
    /// # Errors
    /// World/profile binding and exact lookup failures precede all final output
    /// writes and reader movement. Even transmitted mode requires the selected
    /// history source to belong to this snapshot's world/profile. The cooked result
    /// retains no prediction-history borrow and never mutates the sample owner.
    pub fn commit_from_history(
        self,
        history: PredictionHistorySource<'_>,
    ) -> Result<SnapshotPlan<'storage>, SnapshotError> {
        let epoch = self.message().epoch();
        history
            .check(epoch, self.header().profile())
            .map_err(SnapshotError::PredictionHistory)?;
        if let Some(request) = self.prediction_request() {
            let sample = history
                .resolve(epoch, request)
                .map_err(SnapshotError::PredictionHistory)?;
            self.commit_with_prediction(sample)
        } else {
            self.commit_transmitted()
        }
    }
}
