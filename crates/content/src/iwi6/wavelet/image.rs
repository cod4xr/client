// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    Iwi6WaveletBudget, Iwi6WaveletRequirements, WaveletError, bits::Bits, layout::Layout,
    reconstruct::Level,
};
use crate::iwi6::{Iwi6Format, Iwi6Header, Iwi6Mip};
use std::fmt;

/// Complete reconstructed mip/face views borrowing only caller-published pixels.
/// No allocation, file access, renderer upload or color-space conversion occurs.
pub struct Iwi6WaveletImage<'pixels> {
    bytes: &'pixels [u8],
    layout: Layout,
    consumed: (usize, u8),
    encoded_bytes: usize,
}
impl<'pixels> Iwi6WaveletImage<'pixels> {
    /// Checks header/shape/budgets and returns exact work/output capacities.
    /// Success is metadata admission, without validating the entropy body.
    ///
    /// # Errors
    /// Rejects unsupported shape/encoding, invalid headers and host budget failures.
    pub fn requirements(
        input: &[u8],
        budget: Iwi6WaveletBudget,
    ) -> Result<Iwi6WaveletRequirements, WaveletError> {
        Ok(Layout::parse(input, budget)?.required)
    }
    /// Reconstructs all mips/faces privately, then publishes only after full success.
    /// Work bytes are unspecified on failure; the entire output remains unchanged.
    /// RGB expands to BGRA with opaque alpha; other channel order is preserved.
    /// Like the reviewed decoder, trailing encoded bytes/bits and picmip size fields
    /// are not validated. Consumption metadata exposes that boundary explicitly.
    ///
    /// # Errors
    /// Rejects header/shape/budget/capacity failures and incomplete entropy data.
    pub fn decode(
        input: &[u8],
        budget: Iwi6WaveletBudget,
        scratch: &mut [u8],
        output: &'pixels mut [u8],
    ) -> Result<Self, WaveletError> {
        let layout = Layout::parse(input, budget)?;
        let required = layout.required;
        if scratch.len() < required.scratch_bytes || output.len() < required.decoded_bytes {
            return Err(WaveletError::Capacity);
        }
        let (pixels, delta) =
            scratch[..required.scratch_bytes].split_at_mut(required.decoded_bytes);
        let mut bits = Bits::new(&input[28..]);
        let faces = usize::from(layout.header.shape().face_count());
        for level in (0..layout.header.mip_levels()).rev() {
            let slot = layout.levels[level];
            let (previous, current) = pixels.split_at_mut(slot.offset);
            for face in 0..faces {
                let target = &mut current[face * slot.face_bytes..(face + 1) * slot.face_bytes];
                let source = if level + 1 < layout.header.mip_levels() {
                    let parent = layout.levels[level + 1];
                    let begin = parent.offset + face * parent.face_bytes;
                    &previous[begin..begin + parent.face_bytes]
                } else {
                    &[]
                };
                Level {
                    width: usize::try_from(slot.dimensions.width)
                        .map_err(|_| WaveletError::Geometry)?,
                    height: usize::try_from(slot.dimensions.height)
                        .map_err(|_| WaveletError::Geometry)?,
                    channels: layout.channels,
                    stride: layout.stride,
                }
                .decode(&mut bits, source, target, delta)?;
            }
        }
        output[..required.decoded_bytes].copy_from_slice(pixels);
        Ok(Self {
            bytes: &output[..required.decoded_bytes],
            layout,
            consumed: bits.consumed(),
            encoded_bytes: input.len() - 28,
        })
    }
    /// Returns owned original metadata, independent of the encoded source lifetime.
    #[must_use]
    pub fn header(&self) -> Iwi6Header {
        self.layout.header
    }
    /// Returns the explicit reconstructed dense channel encoding.
    #[must_use]
    pub fn format(&self) -> Iwi6Format {
        self.layout.format
    }
    /// Returns complete decoded byte count, excluding caller tail capacity.
    #[must_use]
    pub fn decoded_bytes(&self) -> usize {
        self.bytes.len()
    }
    /// Returns consumed whole payload bytes and following low-order bits, excluding the header.
    #[must_use]
    pub fn consumed(&self) -> (usize, u8) {
        self.consumed
    }
    /// Returns payload length, which can exceed the consumed entropy extent.
    #[must_use]
    pub fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }
    /// Returns a reconstructed mip that may outlive this plan, borrowing only output.
    #[must_use]
    pub fn mip(&self, level: usize) -> Option<Iwi6Mip<'pixels>> {
        if level >= self.layout.header.mip_levels() {
            return None;
        }
        let slot = self.layout.levels[level];
        let faces = self.layout.header.shape().face_count();
        let end = slot.offset + slot.face_bytes * usize::from(faces);
        Some(Iwi6Mip {
            bytes: &self.bytes[slot.offset..end],
            level,
            faces,
            slot,
        })
    }
}
impl fmt::Debug for Iwi6WaveletImage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Iwi6WaveletImage")
            .field("header", &self.layout.header)
            .field("format", &self.layout.format)
            .field("decoded_bytes", &self.bytes.len())
            .field("consumed", &self.consumed)
            .field("encoded_bytes", &self.encoded_bytes)
            .finish_non_exhaustive()
    }
}
