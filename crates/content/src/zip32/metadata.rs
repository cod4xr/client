// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Zip32Compression;
use std::fmt;

/// Compact copied classic-entry declarations, without names or borrowed archive bytes.
/// Originates only from a checked catalog entry; payload integrity remains unverified.
/// A host must bind offsets to the correct retained archive handle/epoch before I/O.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Zip32EntryMetadata {
    pub(super) data_offset: usize,
    pub(super) compressed_bytes: usize,
    pub(super) uncompressed_bytes: u64,
    pub(super) crc32: u32,
    pub(super) compression: Zip32Compression,
}
impl Zip32EntryMetadata {
    /// Returns the checked compressed span's byte offset in the original archive.
    /// This is classic-profile metadata, not authority to read an arbitrary file.
    #[must_use]
    pub fn data_offset(&self) -> usize {
        self.data_offset
    }
    /// Returns the exact compressed byte count expected from a caller's bounded read.
    #[must_use]
    pub fn compressed_byte_count(&self) -> usize {
        self.compressed_bytes
    }
    /// Returns the declared decoded length, without narrowing engine-facing sizes.
    #[must_use]
    pub fn declared_uncompressed_bytes(&self) -> u64 {
        self.uncompressed_bytes
    }
    /// Returns the declared ZIP CRC32, without claiming payload consistency.
    #[must_use]
    pub fn declared_crc32(&self) -> u32 {
        self.crc32
    }
    /// Returns the reviewed stored/raw-Deflate method.
    #[must_use]
    pub fn compression(&self) -> Zip32Compression {
        self.compression
    }
}
impl fmt::Debug for Zip32EntryMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Zip32EntryMetadata")
            .field("compressed_bytes", &self.compressed_bytes)
            .field("declared_uncompressed_bytes", &self.uncompressed_bytes)
            .field("compression", &self.compression)
            .finish_non_exhaustive()
    }
}
