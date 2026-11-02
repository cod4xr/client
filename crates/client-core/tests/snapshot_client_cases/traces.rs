// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BUDGET, BodyReader, PROFILES, SnapshotClientPlan, Update, dirty, fixture, source, state, values,
};
use std::collections::BTreeMap;

#[test]
fn ordered_map_traces_match_sparse_merge_update_removal_and_zero_reference_rules() {
    for profile in PROFILES {
        let mut model: BTreeMap<u32, u16> = (0..64)
            .filter(|number| number % 3 == 0)
            .map(|number| (number, u16::try_from(number + 1).expect("Value")))
            .collect();
        let mut random = 0x7198_12de_u32;
        for step in 0..128 {
            let previous: Vec<_> = model
                .iter()
                .map(|(&number, &value)| state(number, value))
                .collect();
            let mut updates = Vec::new();
            for number in 0..64 {
                random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                match (random >> 16) % 5 {
                    0 => {
                        updates.push(Update::Remove(number));
                        model.remove(&number);
                    }
                    1 => {
                        updates.push(Update::Retain(number));
                        model.entry(number).or_insert(0);
                    }
                    2 => {
                        let prior = model.get(&number).copied().unwrap_or(0);
                        let value = u16::try_from((random >> 7) & 511).expect("Model value");
                        updates.push(Update::Model(number, prior, value));
                        model.insert(number, value);
                    }
                    _ => {}
                }
            }
            let bytes = fixture(&updates, false, false);
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Trace body");
            let mut output = dirty(64);
            let plan = SnapshotClientPlan::parse(
                &mut reader,
                profile,
                source(&previous),
                BUDGET,
                &mut output,
            )
            .expect("Trace merge");
            let expected: Vec<_> = model
                .iter()
                .map(|(&number, &value)| (number, u32::from(value)))
                .collect();
            assert_eq!(values(&plan), expected, "step {step}");
            assert_eq!(plan.stats().updates, updates.len());
        }
    }
}
