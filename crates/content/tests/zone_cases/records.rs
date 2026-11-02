// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn null_empty_raw_strings_and_serialized_asset_order_remain_distinct() {
    let (body, extent) = body(
        &[None, Some(b""), Some(b"Private%\xff")],
        &[(31, u32::MAX), (3, 0xffff_fffe), (31, 0)],
    );
    let mut cooked = Vec::new();
    let container = checked(&body, extent, &mut cooked);
    let counts = PreparedZone5Directory::counts(&container, BUDGET).expect("Root counts");
    assert_eq!((counts.script_strings, counts.assets), (3, 3));
    let mut scratch_strings = [Zone5StringSlot::default(); 3];
    let mut scratch_assets = [Zone5AssetSlot::default(); 3];
    let plan = PreparedZone5Directory::parse(
        &container,
        BUDGET,
        &mut scratch_strings,
        &mut scratch_assets,
    )
    .expect("Root plan");
    let mut strings = [Zone5StringSlot::default(); 3];
    let mut assets = [Zone5AssetSlot::default(); 3];
    let mut bytes = [0; 32];
    let directory = plan
        .publish(Zone5Output {
            strings: &mut strings,
            assets: &mut assets,
            bytes: &mut bytes,
        })
        .expect("Complete root catalog");
    assert_eq!(directory.script_string(0), Some(None));
    assert_eq!(directory.script_string(1), Some(Some(b"".as_slice())));
    assert_eq!(
        directory.script_string(2),
        Some(Some(b"Private%\xff".as_slice()))
    );
    assert_eq!(directory.script_string(3), None);
    assert_eq!(
        directory
            .assets()
            .map(|asset| asset.kind().code())
            .collect::<Vec<_>>(),
        [31, 3, 31]
    );
    assert_eq!(
        directory.asset(0).expect("record").serialized_link(),
        Zone5SerializedLink::Following
    );
    assert_eq!(
        directory.asset(1).expect("record").serialized_link(),
        Zone5SerializedLink::Insert
    );
    assert_eq!(
        directory.asset(2).expect("record").serialized_link(),
        Zone5SerializedLink::Null
    );
    assert!(directory.asset(3).is_none());
}

#[test]
fn logical_alignment_does_not_skip_physical_input_or_untyped_child_bytes() {
    let (mut body, extent) = body(&[Some(b"a"), None], &[(3, u32::MAX)]);
    assert_eq!((body.len(), extent), (34, 20));
    body.extend_from_slice(b"Opaque child bytes never interpreted");
    let mut cooked = Vec::new();
    let container = checked(&body, extent, &mut cooked);
    let mut strings = [Zone5StringSlot::default(); 2];
    let mut assets = [Zone5AssetSlot::default(); 1];
    let plan = PreparedZone5Directory::parse(
        &container,
        Zone5Budget {
            max_prefix_bytes: 34,
            ..BUDGET
        },
        &mut strings,
        &mut assets,
    )
    .expect("Physical/logical positions differ");
    assert_eq!(plan.metadata().body_prefix_bytes, 34);
    assert_eq!(plan.metadata().virtual_prefix_bytes, 20);
    assert_eq!(plan.metadata().string_bytes, 2);
}

#[test]
fn all_native_kind_labels_and_inert_skipped_link_words_are_preserved() {
    let records: Vec<_> = (0..33)
        .map(|kind| (kind, if kind == 24 { 0x1000_0101 } else { u32::MAX }))
        .collect();
    let (body, extent) = body(&[], &records);
    let mut cooked = Vec::new();
    let container = checked(&body, extent, &mut cooked);
    let mut scratch = [Zone5AssetSlot::default(); 33];
    let plan = PreparedZone5Directory::parse(&container, BUDGET, &mut [], &mut scratch)
        .expect("Every native label and skipped type");
    let mut assets = [Zone5AssetSlot::default(); 33];
    let directory = plan
        .publish(Zone5Output {
            strings: &mut [],
            bytes: &mut [],
            assets: &mut assets,
        })
        .expect("Records");
    for (code, asset) in directory.assets().enumerate() {
        assert_eq!(usize::from(asset.kind().code()), code);
    }
    let skipped = directory.asset(24).expect("Skipped sound-driver record");
    assert_eq!(skipped.kind(), Zone5AssetKind::SoundDriverGlobals);
    // This offset is outside its zero-size block; cataloging must not dereference it.
    assert_eq!(
        skipped.serialized_link(),
        Zone5SerializedLink::Offset {
            block_code: 1,
            offset: 256
        }
    );
}

#[test]
fn packed_link_bias_block_bits_and_unvalidated_namespace_are_explicit() {
    let words = [
        1,
        0x1000_0000,
        0x1000_0001,
        0x8000_0001,
        0xf000_0001,
        0xffff_fffd,
    ];
    let (body, extent) = body(&[], &words.map(|link| (24, link)));
    let mut cooked = Vec::new();
    let container = checked(&body, extent, &mut cooked);
    let mut scratch = [Zone5AssetSlot::default(); 6];
    let plan = PreparedZone5Directory::parse(&container, BUDGET, &mut [], &mut scratch)
        .expect("Inert link fields");
    let mut assets = [Zone5AssetSlot::default(); 6];
    let directory = plan
        .publish(Zone5Output {
            strings: &mut [],
            bytes: &mut [],
            assets: &mut assets,
        })
        .expect("Views");
    let expected = [
        (0, 0),
        (0, 0x0fff_ffff),
        (1, 0),
        (8, 0),
        (15, 0),
        (15, 0x0fff_fffc),
    ];
    for (asset, (block_code, offset)) in directory.assets().zip(expected) {
        assert_eq!(
            asset.serialized_link(),
            Zone5SerializedLink::Offset { block_code, offset }
        );
    }
}

#[test]
fn metadata_counts_do_not_allocate_a_legacy_global_script_string_arena() {
    let (body, extent) = body(&vec![None; 70_000], &[]);
    let mut cooked = Vec::new();
    let container = checked(&body, extent, &mut cooked);
    let mut scratch = vec![Zone5StringSlot::default(); 70_000];
    let plan = PreparedZone5Directory::parse(&container, BUDGET, &mut scratch, &mut [])
        .expect("Host admits wide inert string catalog");
    assert_eq!(plan.metadata().counts.script_strings, 70_000);
    assert_eq!(plan.metadata().null_strings, 70_000);
    assert_eq!(plan.metadata().string_bytes, 0);
    // Typed legacy script references remain a separate, narrower format boundary.
}
