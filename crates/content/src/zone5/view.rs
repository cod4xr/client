// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Zone5AssetRecord, Zone5AssetSlot, Zone5DirectoryMetadata, Zone5StringSlot};
use std::fmt;

/// Published source-independent root catalog over caller-owned cooked storage.
pub struct Zone5Directory<'output> {
    pub(super) strings: &'output [Zone5StringSlot],
    pub(super) assets: &'output [Zone5AssetSlot],
    pub(super) bytes: &'output [u8],
    pub(super) metadata: Zone5DirectoryMetadata,
}
impl<'output> Zone5Directory<'output> {
    /// Returns copied root accounting, without child asset admission.
    #[must_use]
    pub fn metadata(&self) -> Zone5DirectoryMetadata {
        self.metadata
    }
    /// Looks up a script string: outer `None` means invalid index, inner `None`
    /// means explicit null, and an empty byte slice means a real empty string.
    /// Returned views can outlive the directory, borrowing final storage only.
    #[must_use]
    pub fn script_string(&self, index: usize) -> Option<Option<&'output [u8]>> {
        self.strings.get(index).map(|slot| {
            slot.0
                .map(|(offset, length)| &self.bytes[offset..offset + length - 1])
        })
    }
    /// Returns an owned inert asset record that may outlive this catalog.
    #[must_use]
    pub fn asset(&self, index: usize) -> Option<Zone5AssetRecord> {
        self.assets.get(index).map(|slot| slot.0)
    }
    /// Iterates every directory record in serialized order, including skipped types.
    #[must_use]
    pub fn assets(&self) -> impl ExactSizeIterator<Item = Zone5AssetRecord> + '_ {
        self.assets.iter().map(|slot| slot.0)
    }
}
impl fmt::Debug for Zone5Directory<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Zone5Directory")
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
    }
}
