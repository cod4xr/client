// SPDX-License-Identifier: AGPL-3.0-or-later
use super::arguments::Kind;
use super::{InspectError, file, verify};
use cod4xr_client_content::{ArchiveBudget, ArchiveRegistry};
use cod4xr_content::zip32::{Zip32Budget, Zip32Compression, Zip32Directory, Zip32EntrySlot};
use std::{path::Path, time::Instant};

pub(super) fn execute(path: &Path, kind: Kind) -> Result<(), InspectError> {
    let started = Instant::now();
    let verify_payloads = matches!(kind, Kind::VerifiedArchive | Kind::Images);
    let images_only = kind == Kind::Images;
    // Inspection-command resource policy, not engine capacity or negotiated wire limits.
    let budget = Zip32Budget {
        max_archive_bytes: 512 * 1024 * 1024,
        max_directory_bytes: 16 * 1024 * 1024,
        max_entries: 65534,
        max_name_bytes: 4096,
        max_total_name_bytes: 16 * 1024 * 1024,
        max_entry_uncompressed_bytes: 1024 * 1024 * 1024,
        max_total_uncompressed_bytes: 64 * 1024 * 1024 * 1024,
    };
    let mut registry = ArchiveRegistry::new(ArchiveBudget {
        max_open_archives: 1,
        max_archive_bytes: u64::try_from(budget.max_archive_bytes)
            .map_err(|_| InspectError::FileBudget)?,
    });
    let info = registry.open_local_file(path)?;
    let mut input = Vec::new();
    file::resize(
        &mut input,
        usize::try_from(info.bytes()).map_err(|_| InspectError::FileBudget)?,
        0,
    )?;
    registry.read_image(info.id(), &mut input)?;
    let mut scratch = Vec::new();
    let mut output = Vec::new();
    file::resize(&mut scratch, budget.max_entries, Zip32EntrySlot::default())?;
    file::resize(&mut output, budget.max_entries, Zip32EntrySlot::default())?;
    let (stats, stored, directories, metadata) = {
        let archive = Zip32Directory::parse(&input, budget, &mut scratch, &mut output)
            .map_err(InspectError::Archive)?;
        let stats = archive.stats();
        let stored = archive
            .entries()
            .filter(|entry| entry.compression() == Zip32Compression::Stored)
            .count();
        let directories = archive
            .entries()
            .filter(cod4xr_content::zip32::Zip32Entry::has_directory_marker)
            .count();
        let mut metadata = Vec::new();
        if verify_payloads {
            metadata
                .try_reserve_exact(stats.entries)
                .map_err(|_| InspectError::Capacity)?;
            metadata.extend(
                archive
                    .entries()
                    .filter(|entry| {
                        let name = entry.name();
                        !images_only
                            || (name.len() >= 4
                                && name[name.len() - 4..].eq_ignore_ascii_case(b".iwi"))
                    })
                    .map(|entry| entry.metadata()),
            );
        }
        (stats, stored, directories, metadata)
    };
    // Verification retains only copied declarations and the original file handle.
    // Release the whole image before bounded per-entry reads and payload allocations.
    drop(input);
    drop(scratch);
    drop(output);
    let verification = verify_payloads
        .then(|| verify::payloads(&mut registry, info.id(), &metadata, images_only))
        .transpose()?;
    super::archive_report::ArchiveReport {
        started,
        stats,
        stored,
        directories,
        kind,
        verification,
        admitted_source_bytes: registry.usage().admitted_source_bytes,
    }
    .write()
}
