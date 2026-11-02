// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{ArchiveError, ArchiveRegistry};
use cod4xr_content::{
    mount_index::{ArchiveId, MountedAsset},
    zip_payload::{CheckedZipPayload, ZipPayloadBudget, ZipPayloadError},
    zip32::{Zip32Compression, Zip32EntryMetadata},
};

impl ArchiveRegistry {
    /// Reads one complete admitted source into caller private image storage.
    /// The bytes remain untrusted input for a deterministic catalog/import parser.
    /// No final parsed data or archive paths are exposed or applied.
    ///
    /// # Errors
    /// Rejects missing epochs, insufficient/unrepresentable storage, changed source
    /// stamps and I/O. Private image storage is unspecified on failure.
    pub fn read_image<'image>(
        &mut self,
        id: ArchiveId,
        output: &'image mut [u8],
    ) -> Result<&'image [u8], ArchiveError> {
        self.usage.image_attempts = self.usage.image_attempts.saturating_add(1);
        let file = self
            .files
            .iter_mut()
            .find(|file| file.info.id == id)
            .ok_or(ArchiveError::Missing)?;
        let length = usize::try_from(file.info.bytes).map_err(|_| ArchiveError::Bounds)?;
        let output = output
            .get_mut(..length)
            .ok_or(ArchiveError::SourceCapacity)?;
        file.read(0, output)?;
        self.usage.completed_images = self.usage.completed_images.saturating_add(1);
        self.usage.admitted_source_bytes = self
            .usage
            .admitted_source_bytes
            .saturating_add(file.info.bytes);
        Ok(output)
    }

    /// Reads one copied virtual lookup result through its exact retained archive epoch.
    /// Final output is published only after complete stream/length/CRC validation.
    ///
    /// # Errors
    /// Reports the same bounded I/O/storage/validation failures as [`Self::read_entry`].
    pub fn read_asset<'output>(
        &mut self,
        asset: MountedAsset,
        budget: ZipPayloadBudget,
        compressed: &mut [u8],
        scratch: &mut [u8],
        output: &'output mut [u8],
    ) -> Result<CheckedZipPayload<'output>, ArchiveError> {
        self.read_entry(
            asset.archive(),
            asset.metadata(),
            budget,
            compressed,
            scratch,
            output,
        )
    }

    /// Reads exact copied entry metadata from an original owned handle and verifies payloads.
    /// Budgets/capacities preflight before I/O. Private compressed/scratch bytes may change;
    /// every failure preserves final output. The returned payload borrows only final storage.
    /// Host code must associate catalog metadata with its correct original archive ID.
    ///
    /// # Errors
    /// Rejects missing/stale epochs, copied spans outside the admitted source, budgets,
    /// short storage, observable source changes, I/O and complete payload validation.
    pub fn read_entry<'output>(
        &mut self,
        id: ArchiveId,
        metadata: Zip32EntryMetadata,
        budget: ZipPayloadBudget,
        compressed: &mut [u8],
        scratch: &mut [u8],
        output: &'output mut [u8],
    ) -> Result<CheckedZipPayload<'output>, ArchiveError> {
        self.usage.payload_attempts = self.usage.payload_attempts.saturating_add(1);
        let file = self
            .files
            .iter_mut()
            .find(|file| file.info.id == id)
            .ok_or(ArchiveError::Missing)?;
        let count = metadata.compressed_byte_count();
        let count_u64 = u64::try_from(count).map_err(|_| ArchiveError::Bounds)?;
        if count > budget.max_compressed_bytes {
            return Err(ArchiveError::Payload(ZipPayloadError::InputBudget));
        }
        if metadata.declared_uncompressed_bytes() > budget.max_uncompressed_bytes {
            return Err(ArchiveError::Payload(ZipPayloadError::OutputBudget));
        }
        let length = usize::try_from(metadata.declared_uncompressed_bytes())
            .map_err(|_| ArchiveError::Payload(ZipPayloadError::HostSize))?;
        if output.len() < length {
            return Err(ArchiveError::Payload(ZipPayloadError::OutputCapacity));
        }
        if metadata.compression() == Zip32Compression::Deflate
            && scratch.len()
                < length
                    .checked_add(1)
                    .ok_or(ArchiveError::Payload(ZipPayloadError::HostSize))?
        {
            return Err(ArchiveError::Payload(ZipPayloadError::ScratchCapacity));
        }
        let compressed = compressed
            .get_mut(..count)
            .ok_or(ArchiveError::SourceCapacity)?;
        let offset = u64::try_from(metadata.data_offset()).map_err(|_| ArchiveError::Bounds)?;
        file.read(offset, compressed)?;
        let result = self
            .decoder
            .decode_bytes(metadata, compressed, budget, scratch, output)
            .map_err(ArchiveError::Payload)?;
        self.usage.completed_payloads = self.usage.completed_payloads.saturating_add(1);
        self.usage.admitted_source_bytes =
            self.usage.admitted_source_bytes.saturating_add(count_u64);
        self.usage.admitted_payload_bytes = self
            .usage
            .admitted_payload_bytes
            .saturating_add(metadata.declared_uncompressed_bytes());
        Ok(result)
    }
}
