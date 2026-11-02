// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Zip32Budget, Zip32Entry, Zip32EntrySlot, Zip32Error, Zip32Stats, central, end, read};
use std::fmt;

/// Complete checked classic single-disk ZIP catalog over immutable caller bytes.
/// No decompression, checksum validation, file creation or mount order is performed.
pub struct Zip32Directory<'archive, 'storage> {
    archive: &'archive [u8],
    slots: &'storage [Zip32EntrySlot],
    stats: Zip32Stats,
}

impl<'archive, 'storage> Zip32Directory<'archive, 'storage> {
    /// Validates the whole directory and every local span before publishing descriptors.
    /// Supports stored/Deflate, classic signed/unsigned streaming descriptors and inert
    /// unknown extra fields. ZIP64, multi-disk, encryption and other methods reject.
    /// Directory order is preserved even when physical file order differs. Repeated
    /// names remain separate entries; no implicit precedence or path policy is assigned.
    ///
    /// # Errors
    /// Rejects malformed/unsupported records, overlapping local ranges, host budgets and
    /// caller capacities. Final slots remain unchanged on every failure; private scratch
    /// is unspecified. The result borrows only archive bytes and final descriptors.
    pub fn parse(
        archive: &'archive [u8],
        budget: Zip32Budget,
        scratch: &mut [Zip32EntrySlot],
        output: &'storage mut [Zip32EntrySlot],
    ) -> Result<Self, Zip32Error> {
        if archive.len() > budget.max_archive_bytes {
            return Err(Zip32Error::ArchiveBudget);
        }
        let end = end::parse(archive)?;
        if end.directory_bytes > budget.max_directory_bytes {
            return Err(Zip32Error::DirectoryBudget);
        }
        if end.entries > budget.max_entries {
            return Err(Zip32Error::EntryBudget);
        }
        let scratch = scratch
            .get_mut(..end.entries)
            .ok_or(Zip32Error::ScratchCapacity)?;
        if output.len() < end.entries {
            return Err(Zip32Error::OutputCapacity);
        }
        let directory = read::span(archive, end.directory_offset, end.directory_bytes)?;
        let mut stats = Zip32Stats {
            archive_bytes: archive.len(),
            directory_bytes: end.directory_bytes,
            entries: end.entries,
            ..Zip32Stats::default()
        };
        let mut position = 0;
        for (ordinal, target) in scratch.iter_mut().enumerate() {
            let name_budget = budget
                .max_name_bytes
                .min(budget.max_total_name_bytes - stats.name_bytes);
            let (mut slot, next) = central::parse(
                archive,
                end.directory_offset,
                directory,
                position,
                name_budget,
            )?;
            stats.name_bytes = stats
                .name_bytes
                .checked_add(slot.name_bytes)
                .filter(|&length| {
                    length <= budget.max_total_name_bytes
                        && slot.name_bytes <= budget.max_name_bytes
                })
                .ok_or(Zip32Error::NameBudget)?;
            stats.declared_uncompressed_bytes = stats
                .declared_uncompressed_bytes
                .checked_add(u64::from(slot.uncompressed_bytes))
                .filter(|&length| {
                    length <= budget.max_total_uncompressed_bytes
                        && u64::from(slot.uncompressed_bytes) <= budget.max_entry_uncompressed_bytes
                })
                .ok_or(Zip32Error::UncompressedBudget)?;
            stats.compressed_bytes = stats
                .compressed_bytes
                .checked_add(u64::try_from(slot.compressed_bytes).map_err(|_| Zip32Error::Bounds)?)
                .ok_or(Zip32Error::Bounds)?;
            slot.ordinal = ordinal;
            *target = slot;
            position = next;
        }
        if position != directory.len() {
            return Err(Zip32Error::Structure);
        }
        scratch.sort_unstable_by_key(|slot| slot.local_start);
        for pair in scratch.windows(2) {
            if pair[0].local_end > pair[1].local_start {
                return Err(Zip32Error::Overlap);
            }
        }
        scratch.sort_unstable_by_key(|slot| slot.ordinal);
        output[..end.entries].copy_from_slice(scratch);
        Ok(Self {
            archive,
            slots: &output[..end.entries],
            stats,
        })
    }

    /// Iterates original central-directory order, excluding unused caller descriptors.
    #[must_use]
    pub fn entries(&self) -> impl ExactSizeIterator<Item = Zip32Entry<'archive>> + '_ {
        self.slots.iter().map(|&slot| Zip32Entry {
            archive: self.archive,
            slot,
        })
    }
    /// Returns bounded catalog statistics without names, comments or payloads.
    #[must_use]
    pub fn stats(&self) -> Zip32Stats {
        self.stats
    }
}
impl fmt::Debug for Zip32Directory<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Zip32Directory")
            .field("stats", &self.stats)
            .finish_non_exhaustive()
    }
}
