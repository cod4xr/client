// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    ConfigMergeSlot, ConfigPatch, ConfigStateBudget, ConfigStateError, ConfigStateSlot,
    ConfigStateStats, ConfigValue, capture, select,
};
use std::fmt;

/// Complete effective sparse configstrings, before engine interpretation or application.
/// Only cooked storage is borrowed; all source descriptors/values may be released.
pub struct ConfigSnapshot<'storage> {
    bytes: &'storage [u8],
    slots: &'storage [ConfigStateSlot],
    stats: ConfigStateStats,
}
impl<'storage> ConfigSnapshot<'storage> {
    /// Merges immutable defaults and original block-scoped patches without allocation.
    /// Each native block reapplies defaults; nondefault IDs retain their last explicit
    /// value across blocks. No blocks produce an empty effective snapshot. Sparse IDs
    /// never size an array. Scratch contains only private sortable source descriptors.
    ///
    /// # Errors
    /// Rejects input order/block/NUL errors, inspection/retention budgets and caller
    /// capacities before any final writes. Scratch is unspecified on failure; final
    /// text and slots remain unchanged. No strings execute or acquire path authority.
    pub fn build(
        defaults: &[ConfigValue<'_>],
        patches: &[ConfigPatch<'_>],
        budget: ConfigStateBudget,
        scratch: &mut [ConfigMergeSlot],
        output: &'storage mut [u8],
        slots: &'storage mut [ConfigStateSlot],
    ) -> Result<Self, ConfigStateError> {
        let mut stats = capture::capture(defaults, patches, budget, scratch)?;
        let records = &scratch[..stats.input_records];
        for group in select::groups(records) {
            if let Some(record) = select::winner(group, stats.blocks) {
                if stats.records >= budget.max_records {
                    return Err(ConfigStateError::RecordBudget);
                }
                if stats.records >= slots.len() {
                    return Err(ConfigStateError::RecordCapacity);
                }
                let value = select::bytes(record, defaults, patches);
                stats.text_bytes = stats
                    .text_bytes
                    .checked_add(value.len())
                    .filter(|&length| length <= budget.max_text_bytes)
                    .ok_or(ConfigStateError::TextBudget)?;
                if stats.text_bytes > output.len() {
                    return Err(ConfigStateError::OutputCapacity);
                }
                stats.records += 1;
            }
        }
        let mut position = 0;
        let mut offset = 0;
        // Immutable input slices and captured descriptors have no external callbacks.
        // Every selected record, source span and destination extent is proven above.
        for group in select::groups(records) {
            if let Some(record) = select::winner(group, stats.blocks) {
                let value = select::bytes(record, defaults, patches);
                let end = offset + value.len();
                output[offset..end].copy_from_slice(value);
                slots[position] = ConfigStateSlot {
                    index: record.index,
                    offset,
                    length: value.len(),
                };
                offset = end;
                position += 1;
            }
        }
        Ok(Self {
            bytes: &output[..stats.text_bytes],
            slots: &slots[..stats.records],
            stats,
        })
    }
    /// Returns bounded work and retained-resource counts, never source values.
    #[must_use]
    pub fn stats(&self) -> ConfigStateStats {
        self.stats
    }
    /// Looks up one complete effective value without indexing by identifier magnitude.
    #[must_use]
    pub fn value(&self, index: u32) -> Option<&[u8]> {
        self.slots
            .binary_search_by_key(&index, |slot| slot.index)
            .ok()
            .map(|position| self.view(self.slots[position]).bytes())
    }
    /// Iterates sorted unique effective values, excluding unused caller storage.
    #[must_use]
    pub fn records(&self) -> impl ExactSizeIterator<Item = ConfigValue<'_>> {
        self.slots.iter().map(|&slot| self.view(slot))
    }
    fn view(&self, slot: ConfigStateSlot) -> ConfigValue<'_> {
        ConfigValue::new(
            slot.index,
            &self.bytes[slot.offset..slot.offset + slot.length],
        )
    }
}
impl fmt::Debug for ConfigSnapshot<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfigSnapshot")
            .field("stats", &self.stats)
            .finish_non_exhaustive()
    }
}
