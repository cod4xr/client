// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{InspectError, file, report, verify};
use cod4xr_content::zip32::{Zip32Budget, Zip32Compression, Zip32Directory, Zip32EntrySlot};
use std::{path::Path, time::Instant};

pub(super) fn execute(path: &Path, verify_payloads: bool) -> Result<(), InspectError> {
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
    let mut scratch = Vec::new();
    let mut output = Vec::new();
    file::resize(&mut scratch, budget.max_entries, Zip32EntrySlot::default())?;
    file::resize(&mut output, budget.max_entries, Zip32EntrySlot::default())?;
    let archive = Zip32Directory::parse(&input, budget, &mut scratch, &mut output)
        .map_err(InspectError::Archive)?;
    let stats = archive.stats();
    let verification = verify_payloads
        .then(|| verify::payloads(&archive))
        .transpose()?;
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
        payloads_verified = verify_payloads,
        elapsed_micros = started.elapsed().as_micros(),
        "Classic archive inspection completed"
    );
    report::write_value(&serde_json::json!({
        "project": "CoD4XR", "phase": if verify_payloads { "zip32_archive_verification" } else { "zip32_archive_inspection" },
        "archive_bytes": stats.archive_bytes, "directory_bytes": stats.directory_bytes,
        "entries": stats.entries, "name_bytes": stats.name_bytes,
        "compressed_bytes": stats.compressed_bytes,
        "declared_uncompressed_bytes": stats.declared_uncompressed_bytes,
        "stored_entries": stored, "deflate_entries": stats.entries - stored,
        "directory_markers": directories,
        "payload_integrity": if verify_payloads { "crc32_verified" } else { "unverified" },
        "verified_entries": verification.map(|summary| summary.entries),
        "verified_uncompressed_bytes": verification.map(|summary| summary.bytes),
        "gameplay_compatibility": "unverified"
    }))
}
