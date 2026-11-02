// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    Zone5AssetKind, Zone5AssetRecord, Zone5AssetSlot, Zone5Budget, Zone5Counts, Zone5Directory,
    Zone5DirectoryMetadata, Zone5Error, Zone5Output, Zone5StringSlot,
    read::{Reader, word},
    root::Root,
};
use crate::fastfile5::{CheckedFastfile5, Fastfile5Block};
use std::fmt;

/// Complete staged root catalog over immutable source and private caller metadata.
/// Child assets and serialized link targets are not interpreted or followed.
pub struct PreparedZone5Directory<'input, 'scratch> {
    bytes: &'input [u8],
    strings: &'scratch [Zone5StringSlot],
    assets: &'scratch [Zone5AssetSlot],
    metadata: Zone5DirectoryMetadata,
}
impl<'input, 'scratch> PreparedZone5Directory<'input, 'scratch> {
    /// Reads root counts/links and checks minimum physical capacity before allocation.
    /// Success is metadata admission only, not proof of strings or asset records.
    ///
    /// # Errors
    /// Rejects negative counts, unsupported root links, host counts/prefix budgets
    /// and insufficient input even for the minimum pointer/record arrays.
    pub fn counts(
        container: &CheckedFastfile5<'_>,
        budget: Zone5Budget,
    ) -> Result<Zone5Counts, Zone5Error> {
        let mut reader = Reader::new(container.contents(), budget.max_prefix_bytes);
        Ok(Root::parse(&mut reader, budget)?.counts)
    }
    /// Validates every root/string/asset-directory field into private caller cells.
    /// Strings remain exact raw bytes. Only null/following string links are supported;
    /// asset link words remain inert until eventual per-type dispatch.
    /// Physical stream position and logical virtual alignment are distinct.
    /// On failure scratch is unspecified; no final catalog has been published.
    ///
    /// # Errors
    /// Rejects all count errors, short scratch, unsupported script-string aliases,
    /// missing terminators/records, unknown types, host byte budgets and declared
    /// virtual-block overflow. No child payload validation is performed.
    pub fn parse(
        container: &CheckedFastfile5<'input>,
        budget: Zone5Budget,
        strings: &'scratch mut [Zone5StringSlot],
        assets: &'scratch mut [Zone5AssetSlot],
    ) -> Result<Self, Zone5Error> {
        let mut reader = Reader::new(container.contents(), budget.max_prefix_bytes);
        let root = Root::parse(&mut reader, budget)?;
        let counts = root.counts;
        if strings.len() < counts.script_strings || assets.len() < counts.assets {
            return Err(Zone5Error::ScratchCapacity);
        }
        let strings = &mut strings[..counts.script_strings];
        let assets = &mut assets[..counts.assets];
        let pointer_bytes = counts
            .script_strings
            .checked_mul(4)
            .ok_or(Zone5Error::PrefixBudget)?;
        let pointers = reader.take(pointer_bytes)?;
        let start = reader.position();
        let mut null_strings = 0;
        for (slot, pointer) in strings.iter_mut().zip(pointers.chunks_exact(4)) {
            slot.0 = match word(pointer, 0) {
                0 => {
                    null_strings += 1;
                    None
                }
                u32::MAX => {
                    let offset = reader.position() - start;
                    let remaining = budget
                        .max_string_bytes
                        .checked_sub(offset)
                        .ok_or(Zone5Error::StringByteBudget)?;
                    let length = reader.string(remaining)?;
                    Some((offset, length))
                }
                _ => return Err(Zone5Error::StringLink),
            };
        }
        let end = reader.position();
        let records_bytes = counts
            .assets
            .checked_mul(8)
            .ok_or(Zone5Error::PrefixBudget)?;
        let records = reader.take(records_bytes)?;
        for (slot, record) in assets.iter_mut().zip(records.chunks_exact(8)) {
            slot.0 = Zone5AssetRecord {
                kind: Zone5AssetKind::from_word(word(record, 0))?,
                link: word(record, 4),
            };
        }
        let string_bytes = end - start;
        let virtual_prefix_bytes = virtual_extent(
            pointer_bytes,
            string_bytes,
            records_bytes,
            root.allocate_assets,
        )?;
        if virtual_prefix_bytes
            > u64::from(container.metadata().block_bytes(Fastfile5Block::Virtual))
        {
            return Err(Zone5Error::VirtualExtent);
        }
        Ok(Self {
            bytes: &container.contents()[start..end],
            strings,
            assets,
            metadata: Zone5DirectoryMetadata {
                counts,
                null_strings,
                string_bytes,
                body_prefix_bytes: reader.position(),
                virtual_prefix_bytes,
            },
        })
    }
    /// Returns complete root-catalog requirements and accounting without source bytes.
    #[must_use]
    pub fn metadata(&self) -> Zone5DirectoryMetadata {
        self.metadata
    }
    /// Publishes exact metadata/string copies after checking every final capacity.
    /// Every error preserves all output slots and bytes, including unused tails.
    /// Returned views borrow only final output, independently of source and scratch.
    ///
    /// # Errors
    /// Rejects insufficient final string/asset slots or cooked byte capacity.
    pub fn publish<'output>(
        &self,
        output: Zone5Output<'output>,
    ) -> Result<Zone5Directory<'output>, Zone5Error> {
        let Zone5Output {
            strings,
            bytes,
            assets,
        } = output;
        let counts = self.metadata.counts;
        if strings.len() < counts.script_strings
            || assets.len() < counts.assets
            || bytes.len() < self.bytes.len()
        {
            return Err(Zone5Error::OutputCapacity);
        }
        let strings = &mut strings[..counts.script_strings];
        let assets = &mut assets[..counts.assets];
        let bytes = &mut bytes[..self.bytes.len()];
        strings.copy_from_slice(self.strings);
        assets.copy_from_slice(self.assets);
        bytes.copy_from_slice(self.bytes);
        Ok(Zone5Directory {
            strings,
            assets,
            bytes,
            metadata: self.metadata,
        })
    }
}
fn virtual_extent(
    pointers: usize,
    strings: usize,
    records: usize,
    align_assets: bool,
) -> Result<u64, Zone5Error> {
    let mut position = u64::try_from(pointers)
        .ok()
        .and_then(|n| {
            u64::try_from(strings)
                .ok()
                .and_then(|bytes| n.checked_add(bytes))
        })
        .ok_or(Zone5Error::VirtualExtent)?;
    if align_assets {
        position = position.checked_add(3).ok_or(Zone5Error::VirtualExtent)? & !3;
    }
    position
        .checked_add(u64::try_from(records).map_err(|_| Zone5Error::VirtualExtent)?)
        .ok_or(Zone5Error::VirtualExtent)
}
impl fmt::Debug for PreparedZone5Directory<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedZone5Directory")
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
    }
}
