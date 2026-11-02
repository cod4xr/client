// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{InspectError, file, report};
use cod4xr_content::fastfile5::{Fastfile5Budget, Fastfile5Decoder};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};

pub(super) fn execute(path: &Path) -> Result<(), InspectError> {
    let started = Instant::now();
    // Inspector policy, independent from engine capacity and future extended formats.
    let budget = Fastfile5Budget {
        max_file_bytes: 512 * 1024 * 1024,
        max_decoded_bytes: 512 * 1024 * 1024,
        max_total_block_bytes: 8 * 1024 * 1024 * 1024,
        max_external_bytes: 8 * 1024 * 1024 * 1024,
    };
    let input = file::read(path, budget.max_file_bytes)?;
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
    tracing::info!(
        event = "content.inspect.completed",
        profile = "unsigned_pc_fastfile5",
        encoded_bytes = input.len(),
        decoded_bytes = metadata.decoded_bytes(),
        declared_block_bytes = metadata.total_block_bytes(),
        elapsed_micros = started.elapsed().as_micros(),
        "Fastfile container inspection completed"
    );
    report::write_value(&serde_json::json!({
        "project": "CoD4XR", "phase": "fastfile5_inspection",
        "profile": "unsigned_pc_fastfile5", "encoded_bytes": input.len(),
        "decoded_bytes": metadata.decoded_bytes(), "serialized_body_bytes": metadata.body_bytes(),
        "declared_external_bytes": metadata.external_bytes(),
        "declared_total_block_bytes": metadata.total_block_bytes(), "decoded_sha256": digest,
        "container_integrity": "zlib_adler32_verified", "asset_interpretation": "unverified",
        "gameplay_compatibility": "unverified"
    }))
}
