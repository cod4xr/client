// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    ArchiveId, AssetKey, BUDGET, KEY_BUDGET, MountCandidate, MountIndex, MountIndexSlot,
    MountScratchSlot, dirty_slots, keys, metadata,
};

#[test]
fn last_candidate_wins_with_sparse_ids_and_copied_source_independent_lookup() {
    let mut output = [0xa5; 24];
    let mut slots = dirty_slots(5);
    let unused = slots[3..].to_vec();
    let mut scratch = [MountScratchSlot::default(); 6];
    let untouched = scratch[4..].to_vec();
    let index = {
        let names = [&b"B"[..], b"A", b"b", b"raw\x92%"];
        let mut storage: Vec<_> = names.iter().map(|name| vec![0; name.len()]).collect();
        let keys = keys(&names, &mut storage);
        let ids = [u64::MAX, 7, 1, 2];
        let inputs: Vec<_> = keys
            .into_iter()
            .zip(ids)
            .enumerate()
            .map(|(position, (key, id))| {
                MountCandidate::new(
                    key,
                    ArchiveId::new(id),
                    metadata(u32::try_from(position + 1).expect("Small marker")),
                )
            })
            .collect();
        MountIndex::build(&inputs, BUDGET, &mut scratch, &mut output, &mut slots)
            .expect("Copied index")
    };
    assert_eq!(&scratch[4..], &untouched);
    scratch.fill(MountScratchSlot::default());
    assert_eq!(index.stats().candidates, 4);
    assert_eq!(index.stats().files, 3);
    assert_eq!(index.stats().shadowed, 1);
    assert_eq!(index.stats().input_key_bytes, 8);
    assert_eq!(index.stats().key_bytes, 7);
    assert_eq!(
        index
            .records()
            .map(|record| record.key())
            .collect::<Vec<_>>(),
        [&b"a"[..], b"b", b"raw\x92%"]
    );
    for (name, id, marker) in [(&b"A"[..], 7, 2), (b"B", 1, 3), (b"raw\x92%", 2, 4)] {
        let mut query = [0; 8];
        let key = AssetKey::parse(name, KEY_BUDGET, &mut query).expect("Lookup key");
        let asset = index.lookup(key).expect("Retained file");
        assert_eq!(asset.archive().value(), id);
        assert_eq!(asset.metadata().declared_crc32(), marker);
    }
    for name in [&b"0"[..], b"ba", b"z"] {
        let mut query = [0; 2];
        assert!(
            index
                .lookup(AssetKey::parse(name, KEY_BUDGET, &mut query).expect("Missing key"))
                .is_none()
        );
    }
    let debug = format!("{index:?} {:?}", index.records().last().expect("Record"));
    assert!(!debug.contains("raw"));
    assert!(!debug.contains("PrivateSource"));
    assert_eq!(&slots[3..], &unused);
    assert_eq!(&output[7..], &[0xa5; 17]);
}

#[test]
fn returned_asset_routing_survives_index_and_key_storage_release() {
    let expected = metadata(123);
    let asset = {
        let mut name = [0; 1];
        let key = AssetKey::parse(b"x", KEY_BUDGET, &mut name).expect("Key");
        let mut output = [0; 1];
        let mut slots = [MountIndexSlot::default()];
        let index = MountIndex::build(
            &[MountCandidate::new(key, ArchiveId::new(u64::MAX), expected)],
            BUDGET,
            &mut [MountScratchSlot::default()],
            &mut output,
            &mut slots,
        )
        .expect("Index");
        index.lookup(key).expect("Copied routing")
    };
    assert_eq!(asset.archive().value(), u64::MAX);
    assert_eq!(asset.metadata(), expected);
}

#[test]
fn empty_index_and_dirty_workspace_reuse_ignore_unused_source_positions() {
    let empty = MountIndex::build(&[], BUDGET, &mut [], &mut [], &mut []).expect("Empty index");
    assert_eq!(empty.stats().files, 0);
    assert_eq!(empty.records().count(), 0);
    let mut names = vec![vec![0; 1]; 3];
    let keys = keys(&[b"c", b"b", b"a"], &mut names);
    let inputs: Vec<_> = keys
        .into_iter()
        .map(|key| MountCandidate::new(key, ArchiveId::new(1), metadata(1)))
        .collect();
    let mut scratch = [MountScratchSlot::default(); 3];
    MountIndex::build(
        &inputs,
        BUDGET,
        &mut scratch,
        &mut [0; 3],
        &mut dirty_slots(3),
    )
    .expect("Old indices");
    let tail = scratch[1..].to_vec();
    let mut bytes = [0; 1];
    let mut slots = [MountIndexSlot::default()];
    let index = MountIndex::build(&inputs[..1], BUDGET, &mut scratch, &mut bytes, &mut slots)
        .expect("Fresh prefix routing");
    assert_eq!(index.records().next().expect("File").key(), b"c");
    assert_eq!(&scratch[1..], &tail);
}
