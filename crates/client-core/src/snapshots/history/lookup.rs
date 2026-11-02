// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{SnapshotHistory, SnapshotHistoryError, SnapshotHistorySlot};
use crate::snapshots::{SnapshotDelta, SnapshotHeader, SnapshotMessage, SnapshotSource};
use std::cmp::Ordering;

impl<'storage> SnapshotHistory<'_, 'storage> {
    /// Borrows only an exact retained sequence; never substitutes another snapshot.
    /// Lookup uses binary search over the insertion-ordered circular storage.
    #[must_use]
    pub fn source(&self, sequence: u32) -> Option<SnapshotSource<'_>> {
        self.slot(sequence)?.source()
    }
    /// Returns the original opaque header of an exact retained snapshot.
    #[must_use]
    pub fn header(&self, sequence: u32) -> Option<SnapshotHeader> {
        Some(self.slot(sequence)?.retained.as_ref()?.header)
    }
    fn slot(&self, sequence: u32) -> Option<&SnapshotHistorySlot<'storage>> {
        // Committed logical positions are strictly increasing even across the
        // physical wrap. Search exact tags; never index by peer sequence magnitude.
        let oldest = if self.used == self.slots.len() {
            self.next
        } else {
            0
        };
        let (mut low, mut high) = (0, self.used);
        while low < high {
            let middle = low + (high - low) / 2;
            let until_wrap = self.slots.len() - oldest;
            let index = if middle >= until_wrap {
                middle - until_wrap
            } else {
                oldest + middle
            };
            let slot = &self.slots[index];
            let retained = slot.retained.as_ref()?;
            match retained.message.sequence().cmp(&sequence) {
                Ordering::Less => low = middle + 1,
                Ordering::Greater => high = middle,
                Ordering::Equal => return Some(slot),
            }
        }
        None
    }
    /// Resolves a parsed header against the exact current-epoch message distance.
    /// Full snapshots return no prior. All nonzero wire distances remain usable;
    /// missing/evicted entries never trigger implicit full or older fallback.
    ///
    /// # Errors
    /// Rejects epoch/profile/message-order mismatch, nonpositive reference arithmetic
    /// and absent exact references. No state changes or packet parsing occur here.
    pub fn reference(
        &self,
        message: SnapshotMessage,
        header: SnapshotHeader,
    ) -> Result<Option<SnapshotSource<'_>>, SnapshotHistoryError> {
        self.check(message, header.profile())?;
        let SnapshotDelta::Previous(distance) = header.delta() else {
            return Ok(None);
        };
        let sequence = message
            .sequence()
            .checked_sub(u32::from(distance.get()))
            .filter(|&sequence| sequence != 0)
            .ok_or(SnapshotHistoryError::ReferenceSequence)?;
        self.source(sequence)
            .map(Some)
            .ok_or(SnapshotHistoryError::MissingReference)
    }
}
