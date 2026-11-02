// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{PredictionHistoryError, PredictionSlot, PreparedPredictionUpdate};
use crate::{BaselineProtocol, player_state::PredictionSample};
use std::fmt;

/// Exact-time prediction retention in a caller-sized, allocation-free lookup table.
/// Unique keys may arrive in any order, including wrapped native times. World
/// labels must be unique across reconstructed owners; they are not authentication.
pub struct PredictionHistory<'storage> {
    pub(super) slots: &'storage mut [PredictionSlot],
    pub(super) epoch: u64,
    pub(super) profile: BaselineProtocol,
    pub(super) used: usize,
}
impl<'storage> PredictionHistory<'storage> {
    /// Starts a fresh world/profile, clearing old private cells only after validation.
    ///
    /// # Errors
    /// Empty storage rejects without changing any cell.
    pub fn new(
        epoch: u64,
        profile: BaselineProtocol,
        slots: &'storage mut [PredictionSlot],
    ) -> Result<Self, PredictionHistoryError> {
        if slots.is_empty() {
            return Err(PredictionHistoryError::Storage);
        }
        slots.fill(PredictionSlot::default());
        Ok(Self {
            slots,
            epoch,
            profile,
            used: 0,
        })
    }
    /// Clears the old world's samples for an explicit map/connection reset.
    ///
    /// # Errors
    /// Equal/older epochs preserve all metadata and cells.
    pub fn reset(
        &mut self,
        epoch: u64,
        profile: BaselineProtocol,
    ) -> Result<(), PredictionHistoryError> {
        if epoch <= self.epoch {
            return Err(PredictionHistoryError::EpochOrder);
        }
        self.slots.fill(PredictionSlot::default());
        self.epoch = epoch;
        self.profile = profile;
        self.used = 0;
        Ok(())
    }
    /// Returns the explicit host world label, not peer identity.
    #[must_use]
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    /// Returns the current world's native profile.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns caller-funded lookup capacity, independent of the native 128 ring.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
    /// Returns the number of retained distinct exact-time samples.
    #[must_use]
    pub fn len(&self) -> usize {
        self.used
    }
    /// Reports whether this world retains no samples.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.used == 0
    }
    /// Preflights one new exact-time key while exclusively leasing the original owner.
    /// Drop cancels; commit publishes only prechecked cell/count assignments.
    ///
    /// # Errors
    /// World/profile mismatch, duplicates and full storage preserve every cell.
    pub fn prepare_insert<'owner>(
        &'owner mut self,
        epoch: u64,
        sample: PredictionSample,
    ) -> Result<PreparedPredictionUpdate<'owner, 'storage>, PredictionHistoryError> {
        self.check(epoch, sample.profile())?;
        let index = match self.position(sample.command_time()) {
            Ok(_) => return Err(PredictionHistoryError::Duplicate),
            Err(Some(index)) => index,
            Err(None) => return Err(PredictionHistoryError::Capacity),
        };
        Ok(PreparedPredictionUpdate {
            owner: self,
            index,
            sample,
            replacement: false,
        })
    }
    /// Preflights an explicit replay correction for one existing exact-time key.
    /// Does not insert a missing key or choose a nearby prediction.
    ///
    /// # Errors
    /// World/profile mismatch or a missing exact time preserves every cell.
    pub fn prepare_replace<'owner>(
        &'owner mut self,
        epoch: u64,
        sample: PredictionSample,
    ) -> Result<PreparedPredictionUpdate<'owner, 'storage>, PredictionHistoryError> {
        self.check(epoch, sample.profile())?;
        let index = self
            .position(sample.command_time())
            .map_err(|_| PredictionHistoryError::Missing)?;
        Ok(PreparedPredictionUpdate {
            owner: self,
            index,
            sample,
            replacement: true,
        })
    }
    pub(super) fn check(
        &self,
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
impl fmt::Debug for PredictionHistory<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PredictionHistory")
            .field("profile", &self.profile)
            .field("capacity", &self.slots.len())
            .field("retained", &self.used)
            .finish_non_exhaustive()
    }
}
