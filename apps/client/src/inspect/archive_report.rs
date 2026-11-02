// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{InspectError, arguments::Kind, report, verify};
use cod4xr_content::zip32::Zip32Stats;
use std::time::Instant;

pub(super) struct ArchiveReport {
    pub started: Instant,
    pub stats: Zip32Stats,
    pub stored: usize,
    pub directories: usize,
    pub kind: Kind,
    pub verification: Option<verify::Summary>,
    pub admitted_source_bytes: u64,
}
impl ArchiveReport {
    pub fn write(self) -> Result<(), InspectError> {
        let Self {
            started,
            stats,
            stored,
            directories,
            kind,
            verification,
            admitted_source_bytes,
        } = self;
        let images_only = kind == Kind::Images;
        let verify_payloads = verification.is_some();
        let image_summary = verification.and_then(|summary| summary.images);
        tracing::info!(
            event = "content.inspect.completed",
            archive_bytes = stats.archive_bytes,
            entries = stats.entries,
            payloads_verified = verify_payloads,
            selected_images_only = images_only,
            image_headers = image_summary.map_or(0, |summary| summary.headers),
            wavelet_headers = image_summary.map_or(0, |summary| summary.wavelet_headers),
            wavelet_decoded = image_summary.map_or(0, |summary| summary.wavelet_decoded),
            admitted_source_bytes = admitted_source_bytes,
            elapsed_micros = started.elapsed().as_micros(),
            "Classic archive inspection completed"
        );
        report::write_value(&serde_json::json!({
            "project": "CoD4XR", "phase": if images_only { "iwi6_image_inspection" } else if verify_payloads { "zip32_archive_verification" } else { "zip32_archive_inspection" },
            "archive_bytes": stats.archive_bytes, "directory_bytes": stats.directory_bytes,
            "entries": stats.entries, "name_bytes": stats.name_bytes,
            "compressed_bytes": stats.compressed_bytes,
            "declared_uncompressed_bytes": stats.declared_uncompressed_bytes,
            "stored_entries": stored, "deflate_entries": stats.entries - stored,
            "directory_markers": directories,
            "host_admitted_source_bytes": admitted_source_bytes,
            "payload_integrity": if images_only { "selected_image_crc32_verified" } else if verify_payloads { "crc32_verified" } else { "unverified" },
            "verified_entries": verification.map(|summary| summary.entries),
            "verified_uncompressed_bytes": verification.map(|summary| summary.bytes),
            "image_headers": image_summary.map(|summary| summary.headers),
            "direct_image_layouts": image_summary.map(|summary| summary.direct_layouts),
            "wavelet_headers": image_summary.map(|summary| summary.wavelet_headers),
            "wavelet_pixel_decoding_implemented": true,
            "wavelet_decoded_images": image_summary.map(|summary| summary.wavelet_decoded),
            "wavelet_decoded_bytes": image_summary.map(|summary| summary.wavelet_decoded_bytes),
            "wavelet_pixel_sha256": image_summary.and_then(|summary| summary.wavelet_pixel_sha256),
            "wavelet_consumed_bits": image_summary.map(|summary| summary.wavelet_consumed_bits),
            "wavelet_encoded_bytes": image_summary.map(|summary| summary.wavelet_encoded_bytes),
            "image_mip_levels": image_summary.map(|summary| summary.mip_levels),
            "image_texels": image_summary.map(|summary| summary.texels),
            "largest_image_dimension": image_summary.map(|summary| summary.largest_dimension),
            "gameplay_compatibility": "unverified"
        }))
    }
}
