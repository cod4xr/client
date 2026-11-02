// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum Kind {
    #[default]
    Default,
    Patch,
}

/// Opaque caller-owned merge workspace; contents may change on failure.
/// It contains only source indices/counts, never borrowed source values.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConfigMergeSlot {
    pub(super) index: u32,
    pub(super) kind: Kind,
    pub(super) source_position: usize,
    pub(super) block: usize,
}
/// Opaque final sparse metadata, changed only after complete merge preflight.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConfigStateSlot {
    pub(super) index: u32,
    pub(super) offset: usize,
    pub(super) length: usize,
}
