// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded PC zone-root catalogs over container-checked serialized content.
//! Child assets, offset targets, engine memory and renderer resources remain untyped.
mod error;
mod kind;
mod link;
mod plan;
mod read;
mod root;
mod slots;
mod view;

pub use error::Zone5Error;
pub use kind::Zone5AssetKind;
pub use link::Zone5SerializedLink;
pub use plan::PreparedZone5Directory;
pub use slots::{Zone5AssetRecord, Zone5AssetSlot, Zone5StringSlot};
pub use view::Zone5Directory;

/// Independent host admission policy for root metadata and its physical prefix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Zone5Budget {
    /// Maximum script-string entries, including null slots.
    pub max_script_strings: usize,
    /// Maximum asset directory records, independently of child asset sizes.
    pub max_assets: usize,
    /// Maximum contiguous inline string bytes, including NUL terminators.
    pub max_string_bytes: usize,
    /// Maximum consumed serialized body prefix, excluding the fastfile size header.
    pub max_prefix_bytes: usize,
}
/// Copied root counts for exact caller metadata-capacity planning.
/// Success does not validate strings, directory records or child assets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Zone5Counts {
    /// Script-string slots, including nulls and empty strings distinctly.
    pub script_strings: usize,
    /// Asset records in serialized order, without deduplication.
    pub assets: usize,
}
/// Copied complete root-catalog metadata, without child asset admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Zone5DirectoryMetadata {
    /// Root entry counts, independently of engine registries.
    pub counts: Zone5Counts,
    /// Number of explicit null script-string entries.
    pub null_strings: usize,
    /// Cooked inline bytes, including each terminator.
    pub string_bytes: usize,
    /// Consumed body prefix; alignment does not skip physical bytes.
    pub body_prefix_bytes: usize,
    /// Accounted logical virtual-block prefix, including logical alignment.
    pub virtual_prefix_bytes: u64,
}
/// Caller-owned final capacities; all are checked before any publication.
pub struct Zone5Output<'output> {
    /// Opaque final string metadata storage.
    pub strings: &'output mut [Zone5StringSlot],
    /// Final cooked string bytes; unused tail remains unchanged.
    pub bytes: &'output mut [u8],
    /// Opaque final asset metadata storage.
    pub assets: &'output mut [Zone5AssetSlot],
}
