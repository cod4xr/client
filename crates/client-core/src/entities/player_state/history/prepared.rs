// SPDX-License-Identifier: AGPL-3.0-or-later
use super::PredictionHistory;
use crate::player_state::PredictionSample;
use std::fmt;

/// Exclusive original-owner sample update; dropping cancels every cell/count write.
/// Use explicit replacement only when the host has recomputed that exact prediction.
///
/// ```compile_fail,E0499
/// use cod4xr_client_core::player_state::{PredictionSample, history::PredictionHistory};
/// fn original_owner(history: &mut PredictionHistory<'_>, epoch: u64, sample: PredictionSample) {
///     let update = history.prepare_insert(epoch, sample).unwrap();
///     history.reset(epoch + 1, sample.profile()).unwrap();
///     update.commit();
/// }
/// ```
#[must_use = "Commit the preflighted prediction update or drop to cancel."]
pub struct PreparedPredictionUpdate<'owner, 'storage> {
    pub(super) owner: &'owner mut PredictionHistory<'storage>,
    pub(super) index: usize,
    pub(super) sample: PredictionSample,
    pub(super) replacement: bool,
}
impl PreparedPredictionUpdate<'_, '_> {
    /// Reports explicit replacement without exposing native times or player words.
    #[must_use]
    pub fn is_replacement(&self) -> bool {
        self.replacement
    }
    /// Publishes the checked copy with no allocation, lookup, callback or fallible work.
    /// Does not retire inputs, admit network ACKs or advance any simulation clock.
    pub fn commit(self) {
        self.owner.slots[self.index].0 = Some(self.sample);
        if !self.replacement {
            self.owner.used += 1;
        }
    }
}
impl fmt::Debug for PreparedPredictionUpdate<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedPredictionUpdate")
            .field("replacement", &self.replacement)
            .finish_non_exhaustive()
    }
}
