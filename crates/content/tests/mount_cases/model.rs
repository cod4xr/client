// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    ArchiveId, AssetKey, BUDGET, KEY_BUDGET, MountCandidate, MountIndex, MountIndexSlot,
    MountScratchSlot, metadata,
};
use std::collections::BTreeMap;

#[test]
fn canonical_last_insert_model_matches_256_ordered_override_traces() {
    let names = [b"a", b"b", b"c", b"d", b"e", b"f"];
    let mut storage = vec![vec![0; 1]; names.len()];
    let keys: Vec<_> = names
        .iter()
        .zip(&mut storage)
        .map(|(name, bytes)| AssetKey::parse(*name, KEY_BUDGET, bytes).expect("Model key"))
        .collect();
    let declarations: Vec<_> = (0..17).map(metadata).collect();
    for trace in 0..256_usize {
        let mut expected = BTreeMap::new();
        let mut inputs = Vec::new();
        for (position, &declaration) in declarations.iter().enumerate().take(trace % 17) {
            let selected = (trace * 7 + position * 11 + position / 3) % names.len();
            let archive = u64::MAX - u64::try_from((trace + position) % 7).expect("Small model ID");
            let marker = u32::try_from(position).expect("Small marker");
            expected.insert(&names[selected][..], (archive, marker));
            inputs.push(MountCandidate::new(
                keys[selected],
                ArchiveId::new(archive),
                declaration,
            ));
        }
        let mut text = [0; 6];
        let mut slots = [MountIndexSlot::default(); 6];
        let index = MountIndex::build(
            &inputs,
            BUDGET,
            &mut [MountScratchSlot::default(); 17],
            &mut text,
            &mut slots,
        )
        .expect("Model trace");
        let observed: BTreeMap<_, _> = index
            .records()
            .map(|record| {
                (
                    record.key(),
                    (
                        record.asset().archive().value(),
                        record.asset().metadata().declared_crc32(),
                    ),
                )
            })
            .collect();
        assert_eq!(observed, expected);
        assert_eq!(index.stats().files, expected.len());
        assert_eq!(index.stats().shadowed, inputs.len() - expected.len());
        for (&name, &(archive, marker)) in &expected {
            let mut query = [0; 1];
            let key = AssetKey::parse(name, KEY_BUDGET, &mut query).expect("Query");
            let asset = index.lookup(key).expect("Model file");
            assert_eq!(asset.archive().value(), archive);
            assert_eq!(asset.metadata().declared_crc32(), marker);
        }
    }
}
