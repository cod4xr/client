// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{ArchiveRegistry, BUDGET, Fixture, PAYLOAD, declarations, image};
use cod4xr_content::{
    asset_key::{AssetKey, AssetKeyBudget},
    mount_index::{MountCandidate, MountIndex, MountIndexBudget, MountIndexSlot, MountScratchSlot},
};

#[test]
fn copied_lookup_reads_original_owned_handle_after_all_catalog_and_index_storage_is_released() {
    let fixture = Fixture::new(&image(0x3610_a686));
    let mut registry = ArchiveRegistry::new(BUDGET);
    let info = registry
        .open_local_file(&fixture.0)
        .expect("Original handle epoch");
    let asset = {
        let mut source = vec![0; usize::try_from(info.bytes()).expect("Small image")];
        let metadata = declarations(
            registry
                .read_image(info.id(), &mut source)
                .expect("Source image"),
        )[0];
        let mut source_key = [0; 32];
        let key_budget = AssetKeyBudget {
            max_bytes: 32,
            max_components: 1,
        };
        let key = AssetKey::parse(b"PrivateEntry", key_budget, &mut source_key)
            .expect("Canonical file key");
        let mut final_keys = [0; 32];
        let mut slots = [MountIndexSlot::default()];
        let index = MountIndex::build(
            &[MountCandidate::new(key, info.id(), metadata)],
            MountIndexBudget {
                max_candidates: 1,
                max_input_key_bytes: 32,
                max_files: 1,
                max_key_bytes: 32,
            },
            &mut [MountScratchSlot::default()],
            &mut final_keys,
            &mut slots,
        )
        .expect("Copied virtual index");
        let mut query = [0; 32];
        index
            .lookup(AssetKey::parse(b"PRIVATEENTRY", key_budget, &mut query).expect("Query"))
            .expect("Copied route")
    };
    let mut final_output = [0xa5; 8];
    let checked = registry
        .read_asset(asset, PAYLOAD, &mut [0; 7], &mut [0; 6], &mut final_output)
        .expect("Lookup route to retained original file");
    assert!(registry.close(info.id()));
    drop(registry);
    assert_eq!(checked.bytes(), b"hello");
    assert_eq!(&final_output[5..], &[0xa5; 3]);
}
