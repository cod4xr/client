// SPDX-License-Identifier: AGPL-3.0-or-later
//! Copied virtual lookup, native duplicate order and bounded transactional publication.
#![allow(clippy::expect_used, reason = "Finite immutable index fixtures.")]
#[path = "mount_cases/bounds.rs"]
mod bounds;
#[path = "zip_cases/fixture.rs"]
mod fixture;
#[path = "mount_cases/integration.rs"]
mod integration;
#[path = "mount_cases/model.rs"]
mod model;
#[path = "mount_cases/semantics.rs"]
mod semantics;

use cod4xr_content::{
    asset_key::{AssetKey, AssetKeyBudget},
    mount_index::{
        ArchiveId, MountCandidate, MountIndex, MountIndexBudget, MountIndexSlot, MountScratchSlot,
    },
    zip32::{Zip32Budget, Zip32Directory, Zip32EntryMetadata, Zip32EntrySlot},
};

const BUDGET: MountIndexBudget = MountIndexBudget {
    max_candidates: 1024,
    max_input_key_bytes: 1024 * 1024,
    max_files: 1024,
    max_key_bytes: 1024 * 1024,
};
const KEY_BUDGET: AssetKeyBudget = AssetKeyBudget {
    max_bytes: 1024 * 1024,
    max_components: 1024,
};
fn metadata(marker: u32) -> Zip32EntryMetadata {
    let file = fixture::build(
        &[fixture::Spec {
            crc: marker,
            ..fixture::Spec::stored(b"PrivateSource", b"x")
        }],
        &[],
        &[],
    );
    let mut slots = [Zip32EntrySlot::default()];
    let catalog = Zip32Directory::parse(
        &file.bytes,
        Zip32Budget {
            max_archive_bytes: 1024,
            max_directory_bytes: 1024,
            max_entries: 1,
            max_name_bytes: 1024,
            max_total_name_bytes: 1024,
            max_entry_uncompressed_bytes: 1024,
            max_total_uncompressed_bytes: 1024,
        },
        &mut [Zip32EntrySlot::default()],
        &mut slots,
    )
    .expect("Catalog declarations, not verified payloads");
    catalog.entries().next().expect("Entry").metadata()
}
fn keys<'a>(names: &[&[u8]], storage: &'a mut [Vec<u8>]) -> Vec<AssetKey<'a>> {
    names
        .iter()
        .zip(storage)
        .map(|(&name, bytes)| {
            AssetKey::parse(name, KEY_BUDGET, bytes).expect("Canonical source key")
        })
        .collect()
}
fn dirty_slots(count: usize) -> Vec<MountIndexSlot> {
    let mut key_bytes = [0; 3];
    let key = AssetKey::parse(b"old", KEY_BUDGET, &mut key_bytes).expect("Seed key");
    let mut output = [0; 3];
    let mut slots = [MountIndexSlot::default()];
    MountIndex::build(
        &[MountCandidate::new(key, ArchiveId::new(99), metadata(99))],
        BUDGET,
        &mut [MountScratchSlot::default()],
        &mut output,
        &mut slots,
    )
    .expect("Dirty final descriptor");
    vec![slots[0]; count]
}
