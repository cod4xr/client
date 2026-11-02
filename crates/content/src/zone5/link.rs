// SPDX-License-Identifier: AGPL-3.0-or-later
/// Inert serialized link descriptor; never a native pointer or storage authority.
/// Offset fields require per-type interpretation and bounds against actual owned
/// resources. Skipped asset types may contain otherwise invalid offset fields.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Zone5SerializedLink {
    /// Null marker, without a serialized target.
    Null,
    /// Following serialized data, interpreted by the eventual typed loader.
    Following,
    /// Following data with an insertion/alias operation, not performed here.
    Insert,
    /// Packed offset after removing the legacy plus-one bias; still unvalidated.
    Offset {
        /// Raw four-bit block code, which can be outside the nine known blocks.
        block_code: u8,
        /// Raw twenty-eight-bit byte offset, without target/type admission.
        offset: u32,
    },
}
impl Zone5SerializedLink {
    pub(super) fn from_word(word: u32) -> Self {
        match word {
            0 => Self::Null,
            u32::MAX => Self::Following,
            0xffff_fffe => Self::Insert,
            value => {
                let adjusted = value - 1;
                Self::Offset {
                    block_code: adjusted.to_be_bytes()[0] >> 4,
                    offset: adjusted & 0x0fff_ffff,
                }
            }
        }
    }
    pub(super) fn class(self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Following => "following",
            Self::Insert => "insert",
            Self::Offset { .. } => "offset",
        }
    }
}
impl std::fmt::Debug for Zone5SerializedLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Zone5SerializedLink")
            .field("class", &self.class())
            .finish_non_exhaustive()
    }
}
