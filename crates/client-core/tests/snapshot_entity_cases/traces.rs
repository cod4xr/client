// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use std::collections::BTreeMap;

#[test]
fn independent_ordered_map_traces_match_sparse_updates_baseline_fallback_and_removal() {
    for seed in 0..128_u32 {
        let previous: Vec<_> = (0..24)
            .filter(|number| (seed + number) % 3 == 0)
            .map(|number| state(number, 3))
            .collect();
        let baselines: Vec<_> = (0..24)
            .filter(|number| (seed + number) % 4 == 0)
            .map(|number| state(number, 7))
            .collect();
        let old: BTreeMap<_, _> = previous
            .iter()
            .map(|entity| (entity.number(), entity.word(W::Kind)))
            .collect();
        let base: BTreeMap<_, _> = baselines
            .iter()
            .map(|entity| (entity.number(), entity.word(W::Kind)))
            .collect();
        let mut expected = old.clone();
        let mut updates = Vec::new();
        let mut random = seed ^ 0x7654_3210;
        for number in 0..24 {
            random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            match random % 5 {
                0 => {
                    updates.push(Update::Remove(number));
                    expected.remove(&number);
                }
                1 => {
                    updates.push(Update::Retain(number));
                    expected.insert(
                        number,
                        old.get(&number)
                            .or_else(|| base.get(&number))
                            .copied()
                            .unwrap_or(0),
                    );
                }
                2 => {
                    let prior = old
                        .get(&number)
                        .or_else(|| base.get(&number))
                        .copied()
                        .unwrap_or(0);
                    updates.push(Update::Kind(
                        number,
                        u8::try_from(prior).expect("Native type"),
                        13,
                    ));
                    expected.insert(number, 13);
                }
                _ => {}
            }
        }
        let input = fixture(&updates, false, false);
        let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
        let mut output = dirty(32);
        let plan = SnapshotEntityPlan::parse(
            &mut reader,
            EntityDecodeContext::Cod4x21,
            seed,
            sources(&previous, &baselines),
            BUDGET,
            &mut output,
        )
        .expect("Trace");
        assert_eq!(values(&plan), expected.into_iter().collect::<Vec<_>>());
    }
}
