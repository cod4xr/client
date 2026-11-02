// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn every_short_final_capacity_preserves_all_cells_bytes_and_unused_tails() {
    let (body, extent) = body(&[Some(b"Private"), None], &[(3, u32::MAX)]);
    let mut cooked = Vec::new();
    let container = checked(&body, extent, &mut cooked);
    let mut scratch_strings = [Zone5StringSlot::default(); 2];
    let mut scratch_assets = [Zone5AssetSlot::default(); 1];
    let plan = PreparedZone5Directory::parse(
        &container,
        BUDGET,
        &mut scratch_strings,
        &mut scratch_assets,
    )
    .expect("Prepared catalog");
    let mut strings = [Zone5StringSlot::default(); 3];
    let mut assets = [Zone5AssetSlot::default(); 2];
    let mut bytes = [0xa5; 12];
    for (string_count, asset_count, byte_count) in
        [(0, 2, 12), (1, 2, 12), (3, 0, 12), (3, 2, 0), (3, 2, 7)]
    {
        assert_eq!(
            plan.publish(Zone5Output {
                strings: &mut strings[..string_count],
                assets: &mut assets[..asset_count],
                bytes: &mut bytes[..byte_count]
            })
            .err(),
            Some(Zone5Error::OutputCapacity)
        );
        assert_eq!(strings, [Zone5StringSlot::default(); 3]);
        assert_eq!(assets, [Zone5AssetSlot::default(); 2]);
        assert_eq!(bytes, [0xa5; 12]);
    }
    let directory = plan
        .publish(Zone5Output {
            strings: &mut strings,
            assets: &mut assets,
            bytes: &mut bytes,
        })
        .expect("Exact publication after failed retries");
    assert_eq!(
        directory.script_string(0),
        Some(Some(b"Private".as_slice()))
    );
    assert_eq!(directory.script_string(1), Some(None));
    assert_eq!(strings[2], Zone5StringSlot::default());
    assert_eq!(assets[1], Zone5AssetSlot::default());
    assert_eq!(bytes[8..], [0xa5; 4]);
}

#[test]
fn cooked_views_and_owned_records_outlive_container_plan_and_private_scratch() {
    let mut strings = [Zone5StringSlot::default(); 1];
    let mut assets = [Zone5AssetSlot::default(); 1];
    let mut bytes = [0; 8];
    let (text, asset, metadata) = {
        let (body, extent) = body(&[Some(b"Private")], &[(31, u32::MAX)]);
        let mut cooked = Vec::new();
        let container = checked(&body, extent, &mut cooked);
        let mut scratch_strings = [Zone5StringSlot::default(); 1];
        let mut scratch_assets = [Zone5AssetSlot::default(); 1];
        let plan = PreparedZone5Directory::parse(
            &container,
            BUDGET,
            &mut scratch_strings,
            &mut scratch_assets,
        )
        .expect("Prepared");
        assert!(!format!("{plan:?}").contains("Private"));
        let directory = plan
            .publish(Zone5Output {
                strings: &mut strings,
                assets: &mut assets,
                bytes: &mut bytes,
            })
            .expect("Published");
        assert!(!format!("{directory:?}").contains("Private"));
        (
            directory.script_string(0).expect("index").expect("inline"),
            directory.asset(0).expect("record"),
            directory.metadata(),
        )
    };
    assert_eq!(text, b"Private");
    assert_eq!(asset.kind(), Zone5AssetKind::RawFile);
    assert_eq!(metadata.string_bytes, 8);
    assert!(!format!("{asset:?}").contains("4294967295"));
    assert_eq!(asset.serialized_link(), Zone5SerializedLink::Following);
}

#[test]
fn deterministic_root_mutations_are_bounded_and_only_publish_complete_catalogs() {
    let (source, extent) = body(&[Some(b"a"), None], &[(24, 0x1000_0101)]);
    for index in 0..source.len() {
        for value in 0..=255 {
            let mut body = source.clone();
            body[index] = value;
            let mut cooked = Vec::new();
            let container = checked(&body, extent, &mut cooked);
            let mut scratch_strings = [Zone5StringSlot::default(); 2];
            let mut scratch_assets = [Zone5AssetSlot::default(); 1];
            if let Ok(plan) = PreparedZone5Directory::parse(
                &container,
                BUDGET,
                &mut scratch_strings,
                &mut scratch_assets,
            ) {
                let mut strings = [Zone5StringSlot::default(); 2];
                let mut assets = [Zone5AssetSlot::default(); 1];
                let mut bytes = [0; 32];
                let directory = plan
                    .publish(Zone5Output {
                        strings: &mut strings,
                        assets: &mut assets,
                        bytes: &mut bytes,
                    })
                    .expect("Validated capacities");
                let meta = directory.metadata();
                assert!(meta.body_prefix_bytes <= body.len());
                assert!(meta.virtual_prefix_bytes <= u64::from(extent));
                assert_eq!(directory.assets().len(), meta.counts.assets);
                for i in 0..meta.counts.script_strings {
                    assert!(directory.script_string(i).is_some());
                }
            }
        }
    }
}
