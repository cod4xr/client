// SPDX-License-Identifier: AGPL-3.0-or-later
use super::MountedAsset;
use std::fmt;

/// Opaque caller-funded sorting workspace; values have no routing authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MountScratchSlot {
    pub(super) source: usize,
}

/// Opaque caller-funded final descriptor, valid only through a successfully built index.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MountIndexSlot {
    pub(super) data: SlotData,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum SlotData {
    #[default]
    Vacant,
    Occupied {
        offset: usize,
        length: usize,
        asset: MountedAsset,
    },
}
impl MountIndexSlot {
    pub(super) fn record<'a>(&self, bytes: &'a [u8]) -> Option<MountRecord<'a>> {
        match self.data {
            SlotData::Vacant => None,
            SlotData::Occupied {
                offset,
                length,
                asset,
            } => Some(MountRecord {
                key: &bytes[offset..offset + length],
                asset,
            }),
        }
    }
}

/// One canonical file record with copied routing/declarations; no ZIP buffer is borrowed.
#[derive(Clone, Copy)]
pub struct MountRecord<'index> {
    key: &'index [u8],
    asset: MountedAsset,
}
impl<'index> MountRecord<'index> {
    /// Returns canonical key bytes from final index storage.
    #[must_use]
    pub fn key(&self) -> &'index [u8] {
        self.key
    }
    /// Copies routing/declarations for a later host-owned archive read.
    #[must_use]
    pub fn asset(&self) -> MountedAsset {
        self.asset
    }
}
impl fmt::Debug for MountRecord<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MountRecord")
            .field("key_bytes", &self.key.len())
            .field("asset", &self.asset)
            .finish_non_exhaustive()
    }
}
