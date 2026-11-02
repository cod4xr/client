// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{CheckedZipPayload, ZipPayloadBudget, ZipPayloadError};
use crate::zip32::{Zip32Compression, Zip32Entry, Zip32EntryMetadata};
use miniz_oxide::inflate::{
    TINFLStatus,
    core::{DecompressorOxide, decompress, inflate_flags},
};
use std::fmt;

/// Caller-owned reusable inline Deflate state, without hidden output allocation.
/// Every Deflate operation resets the codec; no prior entry's history is retained.
pub struct ZipPayloadDecoder {
    state: DecompressorOxide,
}
impl ZipPayloadDecoder {
    /// Creates fresh inline codec state, without allocating a payload or scratch buffer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: DecompressorOxide::new(),
        }
    }

    /// Checks one catalog entry's complete payload before publishing any final bytes.
    /// Stored entries require no scratch. Deflate requires declared output plus one
    /// guard byte in scratch, so output exceeding its declaration cannot be admitted.
    /// No paths, archives, engine variables or session state are applied.
    ///
    /// # Errors
    /// Rejects host/caller budgets, incomplete/extra Deflate data, declared-length or
    /// CRC mismatches. Final output remains unchanged; scratch/state are unspecified.
    /// Success borrows only final output and leaves unused output tails unchanged.
    pub fn decode<'output>(
        &mut self,
        entry: Zip32Entry<'_>,
        budget: ZipPayloadBudget,
        scratch: &mut [u8],
        output: &'output mut [u8],
    ) -> Result<CheckedZipPayload<'output>, ZipPayloadError> {
        self.decode_bytes(
            entry.metadata(),
            entry.compressed_bytes(),
            budget,
            scratch,
            output,
        )
    }

    /// Checks bytes supplied separately from copied metadata, without retaining a ZIP image.
    /// The source must have exactly the catalog's compressed length. Hosts own archive
    /// handle/epoch binding and bounded file reads; this layer validates bytes only.
    /// All stream/length/CRC rules and rollback guarantees match [`Self::decode`].
    ///
    /// # Errors
    /// Rejects source-length mismatch before decoding, plus all [`Self::decode`] errors.
    pub fn decode_bytes<'output>(
        &mut self,
        metadata: Zip32EntryMetadata,
        input: &[u8],
        budget: ZipPayloadBudget,
        scratch: &mut [u8],
        output: &'output mut [u8],
    ) -> Result<CheckedZipPayload<'output>, ZipPayloadError> {
        if input.len() > budget.max_compressed_bytes
            || metadata.compressed_byte_count() > budget.max_compressed_bytes
        {
            return Err(ZipPayloadError::InputBudget);
        }
        if input.len() != metadata.compressed_byte_count() {
            return Err(ZipPayloadError::InputLength);
        }
        let declared = metadata.declared_uncompressed_bytes();
        if declared > budget.max_uncompressed_bytes {
            return Err(ZipPayloadError::OutputBudget);
        }
        let expected = usize::try_from(declared).map_err(|_| ZipPayloadError::HostSize)?;
        if output.len() < expected {
            return Err(ZipPayloadError::OutputCapacity);
        }
        let decoded = match metadata.compression() {
            Zip32Compression::Stored => input,
            Zip32Compression::Deflate => {
                let capacity = expected.checked_add(1).ok_or(ZipPayloadError::HostSize)?;
                let scratch = scratch
                    .get_mut(..capacity)
                    .ok_or(ZipPayloadError::ScratchCapacity)?;
                self.state.init();
                let (status, consumed, produced) = decompress(
                    &mut self.state,
                    input,
                    scratch,
                    0,
                    inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF,
                );
                if produced > expected {
                    return Err(ZipPayloadError::OutputLength);
                }
                if status != TINFLStatus::Done {
                    return Err(ZipPayloadError::InvalidDeflate);
                }
                if consumed != input.len() {
                    return Err(ZipPayloadError::TrailingInput);
                }
                &scratch[..produced]
            }
        };
        if decoded.len() != expected {
            return Err(ZipPayloadError::OutputLength);
        }
        if crc32fast::hash(decoded) != metadata.declared_crc32() {
            return Err(ZipPayloadError::Checksum);
        }
        output[..expected].copy_from_slice(decoded);
        Ok(CheckedZipPayload {
            bytes: &output[..expected],
        })
    }
}
impl Default for ZipPayloadDecoder {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Debug for ZipPayloadDecoder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ZipPayloadDecoder")
            .finish_non_exhaustive()
    }
}
