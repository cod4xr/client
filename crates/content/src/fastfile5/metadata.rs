// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Fastfile5Budget, Fastfile5Error};

/// Legacy PC size-header block identifiers; these are inert declarations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Fastfile5Block {
    /// Temporary loading storage, index zero.
    Temporary,
    /// Runtime storage, index one.
    Runtime,
    /// Large runtime storage, index two.
    LargeRuntime,
    /// Physical runtime storage, index three.
    PhysicalRuntime,
    /// Virtual serialized storage, index four.
    Virtual,
    /// Large serialized storage, index five.
    Large,
    /// Physical serialized storage, index six.
    Physical,
    /// Vertex serialized storage, index seven.
    Vertex,
    /// Index serialized storage, index eight.
    Index,
}
impl Fastfile5Block {
    const fn index(self) -> usize {
        match self {
            Self::Temporary => 0,
            Self::Runtime => 1,
            Self::LargeRuntime => 2,
            Self::PhysicalRuntime => 3,
            Self::Virtual => 4,
            Self::Large => 5,
            Self::Physical => 6,
            Self::Vertex => 7,
            Self::Index => 8,
        }
    }
}
/// Copied decoded size metadata, without asset validation or allocation authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Fastfile5Metadata {
    body_bytes: u32,
    external_bytes: u32,
    blocks: [u32; 9],
    decoded_bytes: usize,
    scratch_bytes: usize,
    total_block_bytes: u64,
}
impl Fastfile5Metadata {
    pub(super) fn parse(
        header: &[u8; 44],
        budget: Fastfile5Budget,
    ) -> Result<Self, Fastfile5Error> {
        let word = |offset: usize| {
            u32::from_le_bytes([
                header[offset],
                header[offset + 1],
                header[offset + 2],
                header[offset + 3],
            ])
        };
        let body_bytes = word(0);
        let external_bytes = word(4);
        let decoded_bytes = usize::try_from(body_bytes)
            .ok()
            .and_then(|n| n.checked_add(44))
            .filter(|&n| n <= budget.max_decoded_bytes)
            .ok_or(Fastfile5Error::DecodedBudget)?;
        let scratch_bytes = decoded_bytes
            .checked_add(1)
            .ok_or(Fastfile5Error::DecodedBudget)?;
        if u64::from(external_bytes) > budget.max_external_bytes {
            return Err(Fastfile5Error::ExternalBudget);
        }
        let blocks = std::array::from_fn(|index| word(8 + index * 4));
        let total_block_bytes: u64 = blocks.iter().map(|&n| u64::from(n)).sum();
        if total_block_bytes > budget.max_total_block_bytes {
            return Err(Fastfile5Error::BlockBudget);
        }
        Ok(Self {
            body_bytes,
            external_bytes,
            blocks,
            decoded_bytes,
            scratch_bytes,
            total_block_bytes,
        })
    }
    /// Returns declared serialized content bytes after the decoded size header.
    #[must_use]
    pub fn body_bytes(&self) -> u32 {
        self.body_bytes
    }
    /// Returns inert external-size metadata, without interpreting it as file bytes.
    #[must_use]
    pub fn external_bytes(&self) -> u32 {
        self.external_bytes
    }
    /// Returns one inert declared block size; no offset or pointer is resolved.
    #[must_use]
    pub fn block_bytes(&self, block: Fastfile5Block) -> u32 {
        self.blocks[block.index()]
    }
    /// Returns the wide declared block sum, independently of serialized byte counts.
    #[must_use]
    pub fn total_block_bytes(&self) -> u64 {
        self.total_block_bytes
    }
    /// Returns the exact complete decoded section length, including its size header.
    #[must_use]
    pub fn decoded_bytes(&self) -> usize {
        self.decoded_bytes
    }
    /// Returns required private work length: complete decoded bytes plus one guard.
    #[must_use]
    pub fn scratch_bytes(&self) -> usize {
        self.scratch_bytes
    }
}
