// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

/// Opaque caller-funded packed default metadata, initialized with `Default`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DefaultSlot {
    pub(super) index: u32,
    pub(super) offset: usize,
    pub(super) length: usize,
}

/// One raw installed default, with no path/command/engine authority.
#[derive(Clone, Copy)]
pub struct ConfigDefault<'text> {
    pub(super) index: u32,
    pub(super) bytes: &'text [u8],
}
impl ConfigDefault<'_> {
    /// Returns the reviewed baseline default identifier, widened without engine narrowing.
    #[must_use]
    pub fn index(&self) -> u32 {
        self.index
    }
    /// Returns original raw bytes, excluding NUL, with no wire-string normalization.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.bytes
    }
}
impl fmt::Debug for ConfigDefault<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfigDefault")
            .field("index", &self.index)
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
