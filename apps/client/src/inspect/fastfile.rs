// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{InspectError, file, report, zone};
use cod4xr_content::fastfile5::{Fastfile5Budget, Fastfile5Decoder};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};

pub(super) fn execute(path: &Path, inspect_directory: bool) -> Result<(), InspectError> {
    let started = Instant::now();
    // Inspector policy, independent from engine capacity and future extended formats.
    let budget = Fastfile5Budget {
        max_file_bytes: 512 * 1024 * 1024,
        max_decoded_bytes: 512 * 1024 * 1024,
        max_total_block_bytes: 8 * 1024 * 1024 * 1024,
        max_external_bytes: 8 * 1024 * 1024 * 1024,
    };
    let input = file::read(path, budget.max_file_bytes)?;
    let encoded_bytes = input.len();
    let mut decoder = Fastfile5Decoder::new();
    let metadata = decoder
        .inspect(&input, budget)
        .map_err(InspectError::Fastfile)?;
    let mut scratch = Vec::new();
    let mut output = Vec::new();
    file::resize(&mut scratch, metadata.scratch_bytes(), 0)?;
    file::resize(&mut output, metadata.decoded_bytes(), 0)?;
    let checked = decoder
        .decode(&input, budget, &mut scratch, &mut output)
        .map_err(InspectError::Fastfile)?;
    let digest: [u8; 32] = Sha256::digest(checked.decoded_bytes()).into();
    // The checked view borrows final decoded storage only. Release source/work
    // allocations before allocating the independently owned root catalog.
    drop(input);
    drop(scratch);
    let directory = inspect_directory
        .then(|| zone::inspect(&checked))
        .transpose()?;
    tracing::info!(
        event = "content.inspect.completed",
        profile = "unsigned_pc_fastfile5",
        encoded_bytes,
        decoded_bytes = metadata.decoded_bytes(),
        declared_block_bytes = metadata.total_block_bytes(),
        directory_inspected = inspect_directory,
        elapsed_micros = started.elapsed().as_micros(),
        "Fastfile container inspection completed"
    );
    report::write_value(&serde_json::json!({
        "project": "CoD4XR", "phase": if inspect_directory { "zone5_directory_inspection" } else { "fastfile5_inspection" },
        "profile": "unsigned_pc_fastfile5", "encoded_bytes": encoded_bytes,
        "decoded_bytes": metadata.decoded_bytes(), "serialized_body_bytes": metadata.body_bytes(),
        "declared_external_bytes": metadata.external_bytes(),
        "declared_total_block_bytes": metadata.total_block_bytes(), "decoded_sha256": digest,
        "container_integrity": "zlib_adler32_verified",
        "asset_interpretation": if inspect_directory { "root_directory_only" } else { "unverified" },
        "zone_directory": directory.map(|summary| summary.report()),
        "child_asset_interpretation": "unverified",
        "gameplay_compatibility": "unverified"
    }))
}
