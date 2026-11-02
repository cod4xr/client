// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Fastfile5Metadata;
use std::fmt;

/// Complete container-checked decoded bytes, without typed asset or pointer admission.
/// Borrows only caller output; encoded input, work and decoder may be dropped.
pub struct CheckedFastfile5<'output> {
    pub(super) bytes: &'output [u8],
    pub(super) metadata: Fastfile5Metadata,
}
impl<'output> CheckedFastfile5<'output> {
    /// Returns copied size metadata, without allocating declared engine memory.
    #[must_use]
    pub fn metadata(&self) -> Fastfile5Metadata {
        self.metadata
    }
    /// Returns the complete decoded section, including its checked size header.
    #[must_use]
    pub fn decoded_bytes(&self) -> &'output [u8] {
        self.bytes
    }
    /// Returns serialized content after the size header; contents are still untyped.
    #[must_use]
    pub fn contents(&self) -> &'output [u8] {
        &self.bytes[44..]
    }
}
impl fmt::Debug for CheckedFastfile5<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CheckedFastfile5")
            .field("metadata", &self.metadata)
            .field("decoded_bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
