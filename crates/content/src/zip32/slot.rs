// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Zip32Compression;

/// Opaque caller-funded descriptor for private scratch or published metadata.
/// Dirty values have no authority until a successful whole-directory parse.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Zip32EntrySlot {
    pub(super) ordinal: usize,
    pub(super) name_offset: usize,
    pub(super) name_bytes: usize,
    pub(super) local_start: usize,
    pub(super) local_end: usize,
    pub(super) data_offset: usize,
    pub(super) compressed_bytes: usize,
    pub(super) uncompressed_bytes: u32,
    pub(super) crc32: u32,
    pub(super) compression: Zip32Compression,
}
