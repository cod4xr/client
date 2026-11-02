// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Zone5AssetKind, Zone5SerializedLink};
use std::fmt;

/// Copied asset-directory record, without a child-body location or target authority.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Zone5AssetRecord {
    pub(super) kind: Zone5AssetKind,
    pub(super) link: u32,
}
impl Zone5AssetRecord {
    /// Returns the explicit native asset type; no typed child has been loaded.
    #[must_use]
    pub fn kind(self) -> Zone5AssetKind {
        self.kind
    }
    /// Describes the serialized word without following or validating its target.
    /// Apply per-type rules first: native sound-driver records are skipped and
    /// can carry otherwise out-of-bounds words, which remain inert here.
    #[must_use]
    pub fn serialized_link(self) -> Zone5SerializedLink {
        Zone5SerializedLink::from_word(self.link)
    }
}
impl fmt::Debug for Zone5AssetRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Zone5AssetRecord")
            .field("kind", &self.kind)
            .field("link_class", &self.serialized_link().class())
            .finish_non_exhaustive()
    }
}
/// Opaque caller-owned asset metadata cell; meaningful only through a successful plan/view.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Zone5AssetSlot(pub(super) Zone5AssetRecord);
impl Default for Zone5AssetSlot {
    fn default() -> Self {
        Self(Zone5AssetRecord {
            kind: Zone5AssetKind::XModelPieces,
            link: 0,
        })
    }
}
impl fmt::Debug for Zone5AssetSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Zone5AssetSlot").finish_non_exhaustive()
    }
}
/// Opaque caller-owned string metadata cell; null and empty strings stay distinct.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Zone5StringSlot(pub(super) Option<(usize, usize)>);
impl fmt::Debug for Zone5StringSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Zone5StringSlot").finish_non_exhaustive()
    }
}
