// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Zip32Compression, Zip32EntrySlot};
use std::fmt;

/// One bounded catalog entry. Payload integrity, decompression and path policy are separate.
#[derive(Clone, Copy)]
pub struct Zip32Entry<'archive> {
    pub(super) archive: &'archive [u8],
    pub(super) slot: Zip32EntrySlot,
}

impl<'archive> Zip32Entry<'archive> {
    /// Returns original name bytes without decoding, normalization or path authority.
    #[must_use]
    pub fn name(&self) -> &'archive [u8] {
        &self.archive[self.slot.name_offset..self.slot.name_offset + self.slot.name_bytes]
    }
    /// Returns the original central-directory position, preserving duplicate order.
    #[must_use]
    pub fn ordinal(&self) -> usize {
        self.slot.ordinal
    }
    /// Returns the checked compressed span, which still requires decoding/CRC verification.
    #[must_use]
    pub fn compressed_bytes(&self) -> &'archive [u8] {
        &self.archive[self.slot.data_offset..self.slot.data_offset + self.slot.compressed_bytes]
    }
    /// Returns the reviewed stored/Deflate method.
    #[must_use]
    pub fn compression(&self) -> Zip32Compression {
        self.slot.compression
    }
    /// Returns the declared output length as a wide engine-facing size.
    #[must_use]
    pub fn declared_uncompressed_bytes(&self) -> u64 {
        u64::from(self.slot.uncompressed_bytes)
    }
    /// Returns the declared checksum, without claiming payload integrity.
    #[must_use]
    pub fn declared_crc32(&self) -> u32 {
        self.slot.crc32
    }
    /// Reports only the conventional trailing-slash marker, not filesystem attributes.
    #[must_use]
    pub fn has_directory_marker(&self) -> bool {
        self.name().ends_with(b"/")
    }
}
impl fmt::Debug for Zip32Entry<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Zip32Entry")
            .field("ordinal", &self.slot.ordinal)
            .field("name_bytes", &self.slot.name_bytes)
            .field("compressed_bytes", &self.slot.compressed_bytes)
            .field("declared_uncompressed_bytes", &self.slot.uncompressed_bytes)
            .field("compression", &self.slot.compression)
            .finish_non_exhaustive()
    }
}
