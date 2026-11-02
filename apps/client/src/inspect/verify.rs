// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{InspectError, file};
use cod4xr_content::{
    zip_payload::{ZipPayloadBudget, ZipPayloadDecoder, ZipPayloadError},
    zip32::{Zip32Compression, Zip32Directory},
};

#[derive(Clone, Copy)]
pub(super) struct Summary {
    pub entries: usize,
    pub bytes: u64,
}

pub(super) fn payloads(archive: &Zip32Directory<'_, '_>) -> Result<Summary, InspectError> {
    // Verification command policy. Reused buffers grow to the largest admitted entry,
    // never the archive's combined declared output. Nothing is written to disk.
    let budget = ZipPayloadBudget {
        max_compressed_bytes: 128 * 1024 * 1024,
        max_uncompressed_bytes: 64 * 1024 * 1024,
    };
    let mut decoder = ZipPayloadDecoder::new();
    let mut scratch = Vec::new();
    let mut output = Vec::new();
    let mut summary = Summary {
        entries: 0,
        bytes: 0,
    };
    for entry in archive.entries() {
        if entry.compressed_bytes().len() > budget.max_compressed_bytes {
            return Err(InspectError::Payload(ZipPayloadError::InputBudget));
        }
        let length = entry.declared_uncompressed_bytes();
        if length > budget.max_uncompressed_bytes {
            return Err(InspectError::Payload(ZipPayloadError::OutputBudget));
        }
        let length = usize::try_from(length)
            .map_err(|_| InspectError::Payload(ZipPayloadError::HostSize))?;
        file::resize(&mut output, length, 0)?;
        let private_length = if entry.compression() == Zip32Compression::Deflate {
            length
                .checked_add(1)
                .ok_or(InspectError::Payload(ZipPayloadError::HostSize))?
        } else {
            0
        };
        file::resize(&mut scratch, private_length, 0)?;
        decoder
            .decode(entry, budget, &mut scratch, &mut output)
            .map_err(InspectError::Payload)?;
        summary.entries += 1;
        summary.bytes = summary
            .bytes
            .checked_add(entry.declared_uncompressed_bytes())
            .ok_or(InspectError::Payload(ZipPayloadError::HostSize))?;
    }
    Ok(summary)
}
