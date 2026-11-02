// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

/// Caller-owned metadata for one packed value. Construct with `Default`.
/// Dirty/reused slots are overwritten only after complete block validation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConfigStringSlot {
    pub(super) index: u32,
    pub(super) offset: usize,
    pub(super) length: usize,
}

/// One normalized explicit patch, with no command, path or engine authority.
#[derive(Clone, Copy)]
pub struct ConfigString<'text> {
    pub(super) index: u32,
    pub(super) bytes: &'text [u8],
}

impl ConfigString<'_> {
    /// Returns the checked baseline wire identifier without narrowing engine storage.
    #[must_use]
    pub fn index(&self) -> u32 {
        self.index
    }
    /// Returns raw normalized bytes, excluding the wire NUL terminator.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.bytes
    }
}

impl fmt::Debug for ConfigString<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfigString")
            .field("index", &self.index)
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
