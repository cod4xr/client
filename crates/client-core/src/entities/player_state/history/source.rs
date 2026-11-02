// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{PredictionHistoryError, PredictionSlot, lookup::position};
use crate::{
    BaselineProtocol,
    player_state::{PredictionRequest, PredictionSample},
};
use std::fmt;

/// Borrowed exact lookup from one original prediction owner's world/profile.
/// Private fields prevent construction from unrelated cells or peer labels.
///
/// ```compile_fail,E0502
/// use cod4xr_client_core::player_state::{PredictionRequest, history::PredictionHistory};
/// fn original_owner(history: &mut PredictionHistory<'_>, epoch: u64, request: PredictionRequest) {
///     let source = history.source();
///     history.reset(epoch + 1, request.profile()).unwrap();
///     let _ = source.resolve(epoch, request);
/// }
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PredictionHistorySource<'history> {
    pub(super) slots: &'history [PredictionSlot],
    pub(super) epoch: u64,
    pub(super) profile: BaselineProtocol,
}
impl<'history> PredictionHistorySource<'history> {
    /// Returns the original owner's explicit world label, not peer identity.
    #[must_use]
    pub fn epoch(self) -> u64 {
        self.epoch
    }
    /// Returns the original owner's selected native profile.
    #[must_use]
    pub fn profile(self) -> BaselineProtocol {
        self.profile
    }
    /// Resolves only the exact decoded request after checking its world/profile.
    ///
    /// # Errors
    /// Stale worlds, foreign profiles and missing exact times have no fallback.
    pub fn resolve(
        self,
        epoch: u64,
        request: PredictionRequest,
    ) -> Result<&'history PredictionSample, PredictionHistoryError> {
        self.check(epoch, request.profile())?;
        self.sample_at(epoch, request.command_time())
    }
    /// Borrows one exact host-selected time within the original owner's profile.
    ///
    /// # Errors
    /// Stale worlds and missing exact times never substitute another key.
    pub fn sample_at(
        self,
        epoch: u64,
        command_time: u32,
    ) -> Result<&'history PredictionSample, PredictionHistoryError> {
        self.check(epoch, self.profile)?;
        let index =
            position(self.slots, command_time).map_err(|_| PredictionHistoryError::Missing)?;
        self.slots[index]
            .0
            .as_ref()
            .ok_or(PredictionHistoryError::Missing)
    }
    pub(crate) fn check(
        self,
        epoch: u64,
        profile: BaselineProtocol,
    ) -> Result<(), PredictionHistoryError> {
        if epoch != self.epoch {
            return Err(PredictionHistoryError::Epoch);
        }
        if profile != self.profile {
            return Err(PredictionHistoryError::Profile);
        }
        Ok(())
    }
}
impl fmt::Debug for PredictionHistorySource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PredictionHistorySource")
            .field("profile", &self.profile)
            .field("capacity", &self.slots.len())
            .finish_non_exhaustive()
    }
}
