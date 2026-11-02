// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Iwi6Budget, Iwi6Error, Iwi6Header, Iwi6Mip, mip::MipSlot, read};
use std::fmt;

// Version-6 dimensions are u16; floor(log2(65535)) + 1 is sixteen.
// This bounds one format's mip description, not engine image/resource counts.
const MAXIMUM_LEVELS: usize = 16;

/// Complete PC version-6 direct bitmap/BC layout over immutable cooked image bytes.
/// Header metadata alone is insufficient; all active picmip sizes and byte spans validate.
pub struct Iwi6DirectImage<'pixels> {
    input: &'pixels [u8],
    header: Iwi6Header,
    mips: [MipSlot; MAXIMUM_LEVELS],
}
impl<'pixels> Iwi6DirectImage<'pixels> {
    /// Checks complete dense bitmap/BC payload geometry, picmip sizes and exact exhaustion.
    /// Physical order is smallest mip first, with contiguous faces within each mip.
    /// Only used picmip entries are interpreted; inactive entries remain inert metadata.
    ///
    /// # Errors
    /// Rejects header/geometry/budget failures, wavelet encodings, missing/extra data,
    /// size-table mismatches and unrepresentable extents. No allocation, I/O, rendering
    /// or pixel conversion is performed. The input remains unchanged on every result.
    pub fn parse(input: &'pixels [u8], budget: Iwi6Budget) -> Result<Self, Iwi6Error> {
        let header = Iwi6Header::parse(input, budget)?;
        if header.format().is_wavelet() {
            return Err(Iwi6Error::Wavelet);
        }
        if header.mip_levels() > MAXIMUM_LEVELS {
            return Err(Iwi6Error::Dimensions);
        }
        let mut mips = [MipSlot::default(); MAXIMUM_LEVELS];
        let mut offset = 28_usize;
        for level in (0..header.mip_levels()).rev() {
            let dimensions = header
                .dimensions()
                .mip(u32::try_from(level).map_err(|_| Iwi6Error::Extent)?);
            let face_bytes = usize::try_from(header.format().face_bytes(dimensions)?)
                .map_err(|_| Iwi6Error::Extent)?;
            let length = face_bytes
                .checked_mul(usize::from(header.shape().face_count()))
                .ok_or(Iwi6Error::Extent)?;
            read::span(input, offset, length)?;
            let end = offset.checked_add(length).ok_or(Iwi6Error::Extent)?;
            if let Some(&declared) = header.picmip_sizes.get(level)
                && u64::from(declared) != u64::try_from(end).map_err(|_| Iwi6Error::Extent)?
            {
                return Err(Iwi6Error::PicmipSize);
            }
            mips[level] = MipSlot {
                offset,
                face_bytes,
                dimensions,
            };
            offset = end;
        }
        if offset != input.len() {
            return Err(Iwi6Error::TrailingInput);
        }
        Ok(Self {
            input,
            header,
            mips,
        })
    }
    /// Returns copied checked metadata, without pixel bytes or renderer authority.
    #[must_use]
    pub fn header(&self) -> Iwi6Header {
        self.header
    }
    /// Looks up a complete logical mip, highest resolution at level zero.
    /// Returned views may outlive this plan but continue to borrow cooked pixel storage.
    #[must_use]
    pub fn mip(&self, level: usize) -> Option<Iwi6Mip<'pixels>> {
        if level >= self.header.mip_levels() {
            return None;
        }
        Some(self.view(level, self.mips[level]))
    }
    /// Iterates complete logical mips from highest to lowest resolution.
    #[must_use]
    pub fn mips(&self) -> impl ExactSizeIterator<Item = Iwi6Mip<'pixels>> + '_ {
        self.mips[..self.header.mip_levels()]
            .iter()
            .enumerate()
            .map(|(level, &slot)| self.view(level, slot))
    }
    fn view(&self, level: usize, slot: MipSlot) -> Iwi6Mip<'pixels> {
        let faces = self.header.shape().face_count();
        let length = slot.face_bytes * usize::from(faces);
        Iwi6Mip {
            bytes: &self.input[slot.offset..slot.offset + length],
            level,
            faces,
            slot,
        }
    }
}
impl fmt::Debug for Iwi6DirectImage<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Iwi6DirectImage")
            .field("header", &self.header)
            .field("encoded_bytes", &self.input.len())
            .finish_non_exhaustive()
    }
}
