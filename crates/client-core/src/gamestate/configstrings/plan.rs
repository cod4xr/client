// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{ConfigString, ConfigStringBudget, ConfigStringError, ConfigStringSlot, scan};
use crate::{BaselineProtocol, message_body::BodyReader};
use std::fmt;

/// Complete explicit patches in one validated configstring block.
/// This borrows cooked storage only, allowing the source frame to be released.
/// Applying patches/defaults to an engine requires separate whole-gamestate validation.
pub struct ConfigStringBlockPlan<'storage> {
    profile: BaselineProtocol,
    bytes: &'storage [u8],
    slots: &'storage [ConfigStringSlot],
}

impl<'storage> ConfigStringBlockPlan<'storage> {
    /// Parses from immediately after the gamestate `svc_configstring` opcode.
    /// Validates the entire block before writing output/metadata or advancing the
    /// original reader. Private scratch is unspecified on error. No implicit defaults,
    /// automatic engine updates or peer-directed allocations occur.
    ///
    /// # Errors
    /// Rejects counts, identifiers/order, string bounds, truncation and exhausted
    /// host budgets or caller capacities. Final output, slots and reader are unchanged.
    pub fn parse(
        reader: &mut BodyReader<'_>,
        profile: BaselineProtocol,
        budget: ConfigStringBudget,
        scratch: &mut [u8],
        output: &'storage mut [u8],
        slots: &'storage mut [ConfigStringSlot],
    ) -> Result<Self, ConfigStringError> {
        let mut start = reader.clone();
        let count = scan::count(&mut start, profile, budget, slots.len())?;
        let mut validated = start.clone();
        let mut previous = None;
        let mut byte_count = 0;
        for _ in 0..count {
            let (_, length) = scan::entry(&mut validated, profile, &mut previous, scratch)?;
            byte_count = scan::text_size(byte_count, length, budget, output.len())?;
        }

        let mut replay = start;
        let mut previous = None;
        let mut offset = 0;
        for (position, slot) in slots[..count].iter_mut().enumerate() {
            // Identical immutable input, starting cursors, profile and scratch capacity.
            // Preflight proved all reads, lengths and destination extents before writes.
            let (index, length) = scan::entry(&mut replay, profile, &mut previous, scratch)?;
            output[offset..offset + length].copy_from_slice(&scratch[..length]);
            *slot = ConfigStringSlot {
                index,
                offset,
                length,
                first_in_block: position == 0,
            };
            offset += length;
        }
        *reader = validated;
        Ok(Self {
            profile,
            bytes: &output[..byte_count],
            slots: &slots[..count],
        })
    }

    /// Returns the explicit original baseline profile used for validation.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns the number of explicit patches, excluding implicit native defaults.
    #[must_use]
    pub fn record_count(&self) -> usize {
        self.slots.len()
    }
    /// Returns combined normalized bytes, excluding wire NUL terminators.
    #[must_use]
    pub fn text_bytes(&self) -> usize {
        self.bytes.len()
    }
    /// Borrows one checked record; unused metadata and text are never exposed.
    #[must_use]
    pub fn record(&self, position: usize) -> Option<ConfigString<'_>> {
        self.slots.get(position).map(|slot| self.view(*slot))
    }
    /// Iterates all explicit patches in strictly increasing wire-identifier order.
    #[must_use]
    pub fn records(&self) -> impl ExactSizeIterator<Item = ConfigString<'_>> {
        self.slots.iter().map(|&slot| self.view(slot))
    }
    fn view(&self, slot: ConfigStringSlot) -> ConfigString<'_> {
        ConfigString {
            index: slot.index,
            bytes: &self.bytes[slot.offset..slot.offset + slot.length],
            first_in_block: slot.first_in_block,
        }
    }
}

impl fmt::Debug for ConfigStringBlockPlan<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfigStringBlockPlan")
            .field("profile", &self.profile)
            .field("records", &self.slots.len())
            .field("text_bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
