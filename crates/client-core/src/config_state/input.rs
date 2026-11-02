// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

/// Borrowed opaque configstring data; construction grants no engine/path authority.
#[derive(Clone, Copy)]
pub struct ConfigValue<'value> {
    pub(super) index: u32,
    pub(super) bytes: &'value [u8],
}
impl<'value> ConfigValue<'value> {
    /// Describes caller-selected data; the snapshot builder validates budgets/order/NULs.
    #[must_use]
    pub const fn new(index: u32, bytes: &'value [u8]) -> Self {
        Self { index, bytes }
    }
    /// Returns the full sparse identifier, without narrowing or array-index allocation.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }
    /// Returns original raw bytes without normalization or UTF-8 assumptions.
    #[must_use]
    pub const fn bytes(self) -> &'value [u8] {
        self.bytes
    }
}

/// One explicit patch and its original block boundary.
#[derive(Clone, Copy)]
pub struct ConfigPatch<'value> {
    pub(super) value: ConfigValue<'value>,
    pub(super) starts_block: bool,
}
impl<'value> ConfigPatch<'value> {
    /// Describes a patch; first records must mark a block and IDs must ascend within it.
    #[must_use]
    pub const fn new(index: u32, bytes: &'value [u8], starts_block: bool) -> Self {
        Self {
            value: ConfigValue::new(index, bytes),
            starts_block,
        }
    }
}
impl fmt::Debug for ConfigValue<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfigValue")
            .field("index", &self.index)
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
impl fmt::Debug for ConfigPatch<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfigPatch")
            .field("value", &self.value)
            .field("starts_block", &self.starts_block)
            .finish_non_exhaustive()
    }
}
