// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{CheckedFastfile5, Fastfile5Budget, Fastfile5Error, Fastfile5Metadata};
use miniz_oxide::inflate::{
    TINFLStatus,
    core::{DecompressorOxide, decompress, inflate_flags},
};
use std::fmt;

const FLAGS: u32 = inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF
    | inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER;

/// Caller-owned reusable inline zlib state, with no hidden output allocation.
pub struct Fastfile5Decoder {
    state: DecompressorOxide,
}
impl Fastfile5Decoder {
    /// Creates inline codec state without allocating decoded data or work buffers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: DecompressorOxide::new(),
        }
    }
    /// Admits the explicit outer profile and inflates only the size-header prefix.
    /// Success returns requirements, not proof of complete stream or asset validity.
    /// Body decoding resets the codec and validates the full stream independently.
    ///
    /// # Errors
    /// Rejects encoded/decoded/block/external policy failures, mismatched profiles
    /// and missing/invalid compressed size metadata. No caller output is mutated.
    pub fn inspect(
        &mut self,
        input: &[u8],
        budget: Fastfile5Budget,
    ) -> Result<Fastfile5Metadata, Fastfile5Error> {
        let payload = payload(input, budget)?;
        if budget.max_decoded_bytes < 44 {
            return Err(Fastfile5Error::DecodedBudget);
        }
        let mut header = [0; 44];
        self.state.init();
        let (status, _, produced) = decompress(&mut self.state, payload, &mut header, 0, FLAGS);
        if produced != 44 || !matches!(status, TINFLStatus::Done | TINFLStatus::HasMoreOutput) {
            return Err(Fastfile5Error::SizeHeader);
        }
        Fastfile5Metadata::parse(&header, budget)
    }
    /// Validates one complete zlib stream and exact decoded declaration before publication.
    /// Adler-32 integrity, overflow guard and input exhaustion are mandatory.
    /// Every error preserves the entire final output; work/state are unspecified.
    /// Declared memory sizes remain metadata, never allocation or pointer authority.
    ///
    /// # Errors
    /// Rejects all inspection failures, short caller buffers, invalid/truncated streams,
    /// incorrect decoded length, checksum failure and trailing/concatenated streams.
    pub fn decode<'output>(
        &mut self,
        input: &[u8],
        budget: Fastfile5Budget,
        scratch: &mut [u8],
        output: &'output mut [u8],
    ) -> Result<CheckedFastfile5<'output>, Fastfile5Error> {
        let metadata = self.inspect(input, budget)?;
        if output.len() < metadata.decoded_bytes() {
            return Err(Fastfile5Error::OutputCapacity);
        }
        let scratch = scratch
            .get_mut(..metadata.scratch_bytes())
            .ok_or(Fastfile5Error::ScratchCapacity)?;
        self.state.init();
        let payload = &input[12..];
        let (status, consumed, produced) = decompress(&mut self.state, payload, scratch, 0, FLAGS);
        if produced > metadata.decoded_bytes() {
            return Err(Fastfile5Error::OutputLength);
        }
        if status != TINFLStatus::Done {
            return Err(Fastfile5Error::InvalidZlib);
        }
        if consumed != payload.len() {
            return Err(Fastfile5Error::TrailingInput);
        }
        if produced != metadata.decoded_bytes() {
            return Err(Fastfile5Error::OutputLength);
        }
        output[..produced].copy_from_slice(&scratch[..produced]);
        Ok(CheckedFastfile5 {
            bytes: &output[..produced],
            metadata,
        })
    }
}
fn payload(input: &[u8], budget: Fastfile5Budget) -> Result<&[u8], Fastfile5Error> {
    if input.len() > budget.max_file_bytes {
        return Err(Fastfile5Error::FileBudget);
    }
    let header = input.get(..12).ok_or(Fastfile5Error::Truncated)?;
    if &header[..8] != b"IWffu100" || header[8..] != 5_u32.to_le_bytes() {
        return Err(Fastfile5Error::Profile);
    }
    Ok(&input[12..])
}
impl Default for Fastfile5Decoder {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Debug for Fastfile5Decoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fastfile5Decoder").finish_non_exhaustive()
    }
}
