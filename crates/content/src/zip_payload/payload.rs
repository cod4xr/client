// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

/// Whole decoded bytes with exact declared length and ZIP CRC32 verified.
/// CRC consistency establishes no publisher identity or native pure-checksum policy.
pub struct CheckedZipPayload<'output> {
    pub(super) bytes: &'output [u8],
}
impl<'output> CheckedZipPayload<'output> {
    /// Returns complete checked bytes, excluding unused caller storage.
    /// The archive, metadata, decoder and scratch may all be released or reused.
    #[must_use]
    pub fn bytes(&self) -> &'output [u8] {
        self.bytes
    }
}
impl fmt::Debug for CheckedZipPayload<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CheckedZipPayload")
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
