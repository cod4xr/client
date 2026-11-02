// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{PredictionHistory, PredictionHistoryError};
use crate::player_state::{PredictionRequest, PredictionSample};

impl PredictionHistory<'_> {
    /// Borrows only the exact retained request in the explicitly selected world.
    /// No older sample or native prior fallback is selected automatically.
    /// The borrow prevents replacement/reset until the consumer releases it.
    ///
    /// ```compile_fail,E0502
    /// use cod4xr_client_core::player_state::{PredictionRequest, history::PredictionHistory};
    /// fn original_owner(history: &mut PredictionHistory<'_>, epoch: u64, request: PredictionRequest) {
    ///     let sample = history.resolve(epoch, request).unwrap();
    ///     history.reset(epoch + 1, request.profile()).unwrap();
    ///     let _ = sample.words();
    /// }
    /// ```
    ///
    /// # Errors
    /// World/profile mismatch and missing times never substitute another record.
    pub fn resolve(
        &self,
        epoch: u64,
        request: PredictionRequest,
    ) -> Result<&PredictionSample, PredictionHistoryError> {
        self.check(epoch, request.profile())?;
        self.sample_at(epoch, request.command_time())
    }
    /// Borrows an exact raw time within the owner's explicitly checked world/profile.
    /// This is a host lookup, not a native-time-to-input-ID or ACK mapping.
    ///
    /// # Errors
    /// A stale world or absent exact time returns no sample.
    pub fn sample_at(
        &self,
        epoch: u64,
        command_time: u32,
    ) -> Result<&PredictionSample, PredictionHistoryError> {
        self.check(epoch, self.profile)?;
        let index = self
            .position(command_time)
            .map_err(|_| PredictionHistoryError::Missing)?;
        self.slots[index]
            .0
            .as_ref()
            .ok_or(PredictionHistoryError::Missing)
    }
    /// Explicitly removes one exact sample without interpreting ordering or ACKs.
    /// Compacts its probe cluster so repeated removal does not accumulate tombstones.
    ///
    /// # Errors
    /// Stale worlds and missing keys preserve all cells and counters.
    pub fn remove(&mut self, epoch: u64, command_time: u32) -> Result<(), PredictionHistoryError> {
        self.check(epoch, self.profile)?;
        let mut hole = self
            .position(command_time)
            .map_err(|_| PredictionHistoryError::Missing)?;
        self.slots[hole].0 = None;
        let mut scan = self.next(hole);
        // Every other physical cell needs at most one visit, including full tables.
        for _ in 1..self.slots.len() {
            let Some(sample) = self.slots[scan].0 else {
                break;
            };
            let bucket = self.home(sample.command_time());
            if self.distance(bucket, hole) < self.distance(bucket, scan) {
                self.slots[hole].0 = self.slots[scan].0.take();
                hole = scan;
            }
            scan = self.next(scan);
        }
        self.used -= 1;
        Ok(())
    }
    // Ok is the exact occupied key; Err is the first free cell (or full table).
    pub(super) fn position(&self, time: u32) -> Result<usize, Option<usize>> {
        let mut index = self.home(time);
        for _ in 0..self.slots.len() {
            match self.slots[index].0 {
                Some(sample) if sample.command_time() == time => return Ok(index),
                None => return Err(Some(index)),
                Some(_) => index = self.next(index),
            }
        }
        Err(None)
    }
    fn home(&self, time: u32) -> usize {
        // Local engine keys, not a hash of attacker-controlled text or coordinates.
        time.to_le_bytes().into_iter().fold(0_usize, |hash, byte| {
            hash.wrapping_mul(16_777_619) ^ usize::from(byte)
        }) % self.slots.len()
    }
    fn next(&self, index: usize) -> usize {
        if index == self.slots.len() - 1 {
            0
        } else {
            index + 1
        }
    }
    fn distance(&self, start: usize, end: usize) -> usize {
        if end >= start {
            end - start
        } else {
            self.slots.len() - (start - end)
        }
    }
}
