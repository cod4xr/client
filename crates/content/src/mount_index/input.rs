// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{asset_key::AssetKey, zip32::Zip32EntryMetadata};

/// Caller-chosen archive handle identifier, without ownership or file-opening authority.
/// Sparse values retain all bits; hosts must bind them to retained original handles/epochs.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArchiveId(u64);
impl ArchiveId {
    /// Wraps a host routing identifier without opening or authenticating an archive.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    /// Returns the original host identifier without narrowing.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Copied lookup routing and declarations, without borrowed names or archive bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MountedAsset {
    archive: ArchiveId,
    metadata: Zip32EntryMetadata,
}
impl MountedAsset {
    /// Returns the host's archive handle identifier, without granting file authority.
    #[must_use]
    pub fn archive(&self) -> ArchiveId {
        self.archive
    }
    /// Returns copied declarations for a later bounded, checksum-checked payload read.
    #[must_use]
    pub fn metadata(&self) -> Zip32EntryMetadata {
        self.metadata
    }
}

/// One immutable canonical candidate. Later input candidates take precedence.
/// The host supplies explicit archive order and original central order within archives.
#[derive(Clone, Copy, Debug)]
pub struct MountCandidate<'key> {
    key: AssetKey<'key>,
    asset: MountedAsset,
}
impl<'key> MountCandidate<'key> {
    /// Associates checked virtual bytes and catalog metadata with a host archive ID.
    /// This routes data only; it does not retain/open a handle or apply pure policy.
    #[must_use]
    pub fn new(key: AssetKey<'key>, archive: ArchiveId, metadata: Zip32EntryMetadata) -> Self {
        Self {
            key,
            asset: MountedAsset { archive, metadata },
        }
    }
    /// Returns the canonical key without retaining a temporary candidate view.
    #[must_use]
    pub fn key(&self) -> AssetKey<'key> {
        self.key
    }
    /// Returns copied archive routing and entry declarations.
    #[must_use]
    pub fn asset(&self) -> MountedAsset {
        self.asset
    }
}
