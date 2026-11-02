// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Iwi6Dimensions;
use std::fmt;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct MipSlot {
    pub offset: usize,
    pub face_bytes: usize,
    pub dimensions: Iwi6Dimensions,
}
/// One complete direct bitmap/BC mip with all its contiguous faces.
/// Wavelet decoding returns these same views over reconstructed dense pixels.
/// Pixel conversion, cube axis orientation and backend uploads are separate operations.
#[derive(Clone, Copy)]
pub struct Iwi6Mip<'pixels> {
    pub(super) bytes: &'pixels [u8],
    pub(super) level: usize,
    pub(super) faces: u8,
    pub(super) slot: MipSlot,
}
impl<'pixels> Iwi6Mip<'pixels> {
    /// Returns logical level zero for base resolution, independent of physical file order.
    #[must_use]
    pub fn level(&self) -> usize {
        self.level
    }
    /// Returns effective dimensions for one face/volume at this level.
    #[must_use]
    pub fn dimensions(&self) -> Iwi6Dimensions {
        self.slot.dimensions
    }
    /// Returns the number of encoded faces, without assigning cube axis names.
    #[must_use]
    pub fn face_count(&self) -> u8 {
        self.faces
    }
    /// Returns complete encoded bytes across all faces/slices at this level.
    #[must_use]
    pub fn bytes(&self) -> &'pixels [u8] {
        self.bytes
    }
    /// Borrows one complete face; volume images have one face containing all depth slices.
    #[must_use]
    pub fn face(&self, index: u8) -> Option<&'pixels [u8]> {
        if index >= self.faces {
            return None;
        }
        let begin = usize::from(index) * self.slot.face_bytes;
        Some(&self.bytes[begin..begin + self.slot.face_bytes])
    }
}
impl fmt::Debug for Iwi6Mip<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Iwi6Mip")
            .field("level", &self.level)
            .field("dimensions", &self.slot.dimensions)
            .field("faces", &self.faces)
            .field("encoded_bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
