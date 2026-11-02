// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    MountCandidate, MountIndexBudget, MountIndexError, MountIndexSlot, MountIndexStats,
    MountRecord, MountScratchSlot, MountedAsset, slot::SlotData,
};
use crate::asset_key::AssetKey;
use std::{cmp::Ordering, fmt};

/// Sorted canonical file lookup borrowing only final copied keys and descriptors.
/// Archive inputs, candidates, source keys and private scratch may all be released.
pub struct MountIndex<'storage> {
    keys: &'storage [u8],
    slots: &'storage [MountIndexSlot],
    stats: MountIndexStats,
}
impl<'storage> MountIndex<'storage> {
    /// Selects the last candidate for each canonical key in caller-specified order.
    /// This preserves native within-archive duplicate precedence when the host supplies
    /// original central order. Archive priority is explicit input order, never ID order.
    /// No archive handle, OS path, pure policy or payload data is admitted/applied here.
    ///
    /// # Errors
    /// Rejects host/caller budgets before final writes. Private scratch is unspecified
    /// on failure; final key bytes and descriptors remain unchanged. Inputs are immutable
    /// slices, so sort/preflight/publication replay no external iterators or callbacks.
    pub fn build(
        candidates: &[MountCandidate<'_>],
        budget: MountIndexBudget,
        scratch: &mut [MountScratchSlot],
        keys: &'storage mut [u8],
        slots: &'storage mut [MountIndexSlot],
    ) -> Result<Self, MountIndexError> {
        if candidates.len() > budget.max_candidates {
            return Err(MountIndexError::CandidateBudget);
        }
        let scratch = scratch
            .get_mut(..candidates.len())
            .ok_or(MountIndexError::ScratchCapacity)?;
        let mut stats = MountIndexStats {
            candidates: candidates.len(),
            ..MountIndexStats::default()
        };
        for (source, (candidate, target)) in candidates.iter().zip(scratch.iter_mut()).enumerate() {
            stats.input_key_bytes = stats
                .input_key_bytes
                .checked_add(candidate.key().bytes().len())
                .filter(|&length| length <= budget.max_input_key_bytes)
                .ok_or(MountIndexError::InputKeyBudget)?;
            target.source = source;
        }
        scratch.sort_unstable_by(|left, right| {
            candidates[left.source]
                .key()
                .bytes()
                .cmp(candidates[right.source].key().bytes())
                .then_with(|| left.source.cmp(&right.source))
        });
        for candidate in winners(candidates, scratch) {
            if stats.files >= budget.max_files {
                return Err(MountIndexError::FileBudget);
            }
            if stats.files >= slots.len() {
                return Err(MountIndexError::SlotCapacity);
            }
            stats.key_bytes = stats
                .key_bytes
                .checked_add(candidate.key().bytes().len())
                .filter(|&length| length <= budget.max_key_bytes)
                .ok_or(MountIndexError::KeyBudget)?;
            if stats.key_bytes > keys.len() {
                return Err(MountIndexError::OutputCapacity);
            }
            stats.files += 1;
        }
        stats.shadowed = stats.candidates - stats.files;
        let mut offset = 0;
        // Immutable candidate/key views and captured indices preserve every proven
        // source/destination bound across this callback-free publication pass.
        for (position, candidate) in winners(candidates, scratch).enumerate() {
            let key = candidate.key().bytes();
            let end = offset + key.len();
            keys[offset..end].copy_from_slice(key);
            slots[position] = MountIndexSlot {
                data: SlotData::Occupied {
                    offset,
                    length: key.len(),
                    asset: candidate.asset(),
                },
            };
            offset = end;
        }
        Ok(Self {
            keys: &keys[..stats.key_bytes],
            slots: &slots[..stats.files],
            stats,
        })
    }
    /// Looks up a validated canonical file key, copying only archive routing/metadata.
    /// A returned value does not borrow the index, acquire a handle or perform I/O.
    #[must_use]
    pub fn lookup(&self, key: AssetKey<'_>) -> Option<MountedAsset> {
        let mut low = 0;
        let mut high = self.slots.len();
        while low < high {
            let middle = low + (high - low) / 2;
            let record = self.slots[middle].record(self.keys)?;
            match record.key().cmp(key.bytes()) {
                Ordering::Less => low = middle + 1,
                Ordering::Greater => high = middle,
                Ordering::Equal => return Some(record.asset()),
            }
        }
        None
    }
    /// Iterates retained files in canonical byte order, excluding unused caller storage.
    pub fn records(&self) -> impl Iterator<Item = MountRecord<'_>> {
        self.slots.iter().filter_map(|slot| slot.record(self.keys))
    }
    /// Returns bounded work/retention counts without private names or archive IDs.
    #[must_use]
    pub fn stats(&self) -> MountIndexStats {
        self.stats
    }
}
fn winners<'a, 'key>(
    candidates: &'a [MountCandidate<'key>],
    scratch: &'a [MountScratchSlot],
) -> impl Iterator<Item = &'a MountCandidate<'key>> {
    scratch
        .chunk_by(|a, b| candidates[a.source].key().bytes() == candidates[b.source].key().bytes())
        .filter_map(|group| group.last().map(|slot| &candidates[slot.source]))
}
impl fmt::Debug for MountIndex<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MountIndex")
            .field("stats", &self.stats)
            .finish_non_exhaustive()
    }
}
