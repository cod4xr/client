// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{InspectError, file, report};
use cod4xr_content::zip32::{Zip32Budget, Zip32Compression, Zip32Directory, Zip32EntrySlot};
use std::{path::Path, time::Instant};

pub(super) fn execute(path: &Path) -> Result<(), InspectError> {
    let started = Instant::now();
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
    let input = file::read(path, budget.max_archive_bytes)?;
    let mut scratch = vec![Zip32EntrySlot::default(); budget.max_entries];
    let mut output = vec![Zip32EntrySlot::default(); budget.max_entries];
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
    tracing::info!(
        event = "content.inspect.completed",
        archive_bytes = stats.archive_bytes,
        entries = stats.entries,
        elapsed_micros = started.elapsed().as_micros(),
        "Classic archive metadata cataloged"
    );
    report::write_value(&serde_json::json!({
        "project": "CoD4XR", "phase": "zip32_archive_inspection",
        "archive_bytes": stats.archive_bytes, "directory_bytes": stats.directory_bytes,
        "entries": stats.entries, "name_bytes": stats.name_bytes,
        "compressed_bytes": stats.compressed_bytes,
        "declared_uncompressed_bytes": stats.declared_uncompressed_bytes,
        "stored_entries": stored, "deflate_entries": stats.entries - stored,
        "directory_markers": directories,
        "payload_integrity": "unverified", "gameplay_compatibility": "unverified"
    }))
}
