// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, dirty_slots};
use cod4xr_client_core::config_state::{ConfigMergeSlot, ConfigPatch, ConfigSnapshot, ConfigValue};
use std::collections::BTreeMap;

#[test]
fn sparse_merge_matches_independent_native_block_reapplication_model() {
    let values = [&b""[..], &b"a%"[..], &b"\x92"[..], &b"longer"[..]];
    for seed in 0_u32..256 {
        let default_ids: Vec<_> = (0_u32..16)
            .filter(|index| (index + seed) % 3 == 0)
            .collect();
        let defaults: Vec<_> = default_ids
            .iter()
            .map(|&index| {
                ConfigValue::new(index, values[usize::try_from(index % 4).expect("Value")])
            })
            .collect();
        let mut patches = vec![];
        let mut expected = BTreeMap::new();
        let block_count = seed % 9;
        for block in 0..block_count {
            for value in &defaults {
                expected.insert(value.index(), value.bytes().to_vec());
            }
            let mut first = true;
            for index in 0_u32..24 {
                if (index.wrapping_mul(7) + block * 3 + seed) % 5 == 0 {
                    let value = values[usize::try_from((seed + index + block) % 4).expect("Value")];
                    patches.push(ConfigPatch::new(index, value, first));
                    expected.insert(index, value.to_vec());
                    first = false;
                }
            }
            assert!(!first); // Every native block has a positive explicit count.
        }
        let mut scratch = vec![ConfigMergeSlot::default(); defaults.len() + patches.len()];
        let mut output = [0; 256];
        let mut slots = dirty_slots(24);
        let result = ConfigSnapshot::build(
            &defaults,
            &patches,
            BUDGET,
            &mut scratch,
            &mut output,
            &mut slots,
        )
        .expect("Bounded model trace");
        let actual: BTreeMap<_, _> = result
            .records()
            .map(|value| (value.index(), value.bytes().to_vec()))
            .collect();
        assert_eq!(actual, expected, "Seed {seed}");
        assert_eq!(
            result.stats().blocks,
            usize::try_from(block_count).expect("Block count")
        );
        assert_eq!(result.stats().records, actual.len());
    }
}
