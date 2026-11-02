// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller-ordered virtual file lookup over copied keys and numeric archive metadata.
//! Host archive handles/epochs, I/O, pure policy and engine application remain separate.

mod error;
mod index;
mod input;
mod slot;

pub use error::MountIndexError;
pub use index::MountIndex;
pub use input::{ArchiveId, MountCandidate, MountedAsset};
pub use slot::{MountIndexSlot, MountRecord, MountScratchSlot};

/// Host inspection/retention budgets, independent of archive identifiers or engine arenas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MountIndexBudget {
    /// Maximum immutable input candidates, including duplicates.
    pub max_candidates: usize,
    /// Maximum combined input key bytes, including shadowed candidates.
    pub max_input_key_bytes: usize,
    /// Maximum unique canonical files retained.
    pub max_files: usize,
    /// Maximum combined retained canonical key bytes.
    pub max_key_bytes: usize,
}
/// Inspection/retention counts without names, payloads or archive identifiers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MountIndexStats {
    /// Inspected immutable candidates.
    pub candidates: usize,
    /// Combined inspected key bytes, including duplicates.
    pub input_key_bytes: usize,
    /// Retained unique canonical files.
    pub files: usize,
    /// Packed retained canonical key bytes.
    pub key_bytes: usize,
    /// Earlier candidates superseded by later canonical duplicates.
    pub shadowed: usize,
}
