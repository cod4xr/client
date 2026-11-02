// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

/// Inert normalized token with its reviewed Windows native copy boundary.
/// Full text remains available for future explicit extension policy. Neither view
/// is a trusted path, translation key, console command or printable log value.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PreGameText<'command> {
    bytes: &'command [u8],
    legacy_limit: usize,
}
impl<'command> PreGameText<'command> {
    pub(super) fn new(bytes: &'command [u8], legacy_limit: usize) -> Self {
        Self {
            bytes,
            legacy_limit,
        }
    }
    /// Borrows the complete normalized token, without UTF-8 conversion or truncation.
    #[must_use]
    pub fn as_bytes(self) -> &'command [u8] {
        self.bytes
    }
    /// Borrows the prefix copied by the reviewed native pre-game handler, without NUL.
    /// Map/gametype fields use 255 bytes; disconnect reason fields use 1023 bytes.
    #[must_use]
    pub fn legacy_bytes(self) -> &'command [u8] {
        &self.bytes[..self.bytes.len().min(self.legacy_limit)]
    }
    /// Reports whether the reviewed native copy would truncate this token.
    #[must_use]
    pub fn legacy_was_truncated(self) -> bool {
        self.bytes.len() > self.legacy_limit
    }
}
impl fmt::Debug for PreGameText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreGameText")
            .field("bytes", &self.bytes.len())
            .field("legacy_limit", &self.legacy_limit)
            .finish_non_exhaustive()
    }
}
