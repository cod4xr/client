// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    PredictionHistory, PredictionHistoryError as Error, PredictionSlot, prediction, profiles,
};
use std::collections::BTreeMap;

#[test]
fn exact_sample_table_matches_independent_ordered_map_through_full_clusters_and_replay() {
    for profile in profiles() {
        for capacity in [1, 2, 3, 17, 257] {
            let mut slots = vec![PredictionSlot::default(); capacity];
            let mut history = PredictionHistory::new(7, profile, &mut slots).expect("Owner");
            let mut model = BTreeMap::new();
            // Fill first so every capacity exercises full-table misses and deletion.
            for time in (0_u32..).take(capacity) {
                super::insert(&mut history, time, time);
                model.insert(time, time);
            }
            let mut random = 0x1234_5678_u32;
            for step in 0..2048_u32 {
                random ^= random << 13;
                random ^= random >> 17;
                random ^= random << 5;
                let time = random % 521;
                match (random >> 16) % 7 {
                    0 | 1 => retain(&mut history, &mut model, time, step, random & 0x100 != 0),
                    2 | 3 => replace(&mut history, &mut model, time, step, random & 0x100 != 0),
                    4 | 5 => {
                        let expected = model.remove(&time).map(|_| ()).ok_or(Error::Missing);
                        assert_eq!(history.remove(history.epoch(), time), expected);
                    }
                    _ => {
                        let expected = model
                            .get(&time)
                            .map(|&value| prediction(profile, time, value))
                            .ok_or(Error::Missing);
                        assert_eq!(history.sample_at(history.epoch(), time).copied(), expected);
                    }
                }
                assert_eq!(history.len(), model.len());
                for (&key, &value) in &model {
                    assert_eq!(
                        history.sample_at(history.epoch(), key),
                        Ok(&prediction(profile, key, value))
                    );
                }
                if step == 1023 {
                    history.reset(8, profile).expect("Fresh world");
                    model.clear();
                }
            }
        }
    }
}
fn retain(
    history: &mut PredictionHistory<'_>,
    model: &mut BTreeMap<u32, u32>,
    time: u32,
    value: u32,
    commit: bool,
) {
    let sample = prediction(history.profile(), time, value);
    let expected = if model.contains_key(&time) {
        Err(Error::Duplicate)
    } else if model.len() == history.capacity() {
        Err(Error::Capacity)
    } else {
        Ok(())
    };
    let result = history.prepare_insert(history.epoch(), sample);
    if let Err(error) = expected {
        assert_eq!(result.expect_err("Rejected without mutation"), error);
    } else {
        let prepared = result.expect("Preflight");
        if commit {
            prepared.commit();
            model.insert(time, value);
        }
    }
}
fn replace(
    history: &mut PredictionHistory<'_>,
    model: &mut BTreeMap<u32, u32>,
    time: u32,
    value: u32,
    commit: bool,
) {
    let sample = prediction(history.profile(), time, value);
    let result = history.prepare_replace(history.epoch(), sample);
    if let Some(previous) = model.get_mut(&time) {
        let prepared = result.expect("Existing exact key");
        if commit {
            prepared.commit();
            *previous = value;
        }
    } else {
        assert_eq!(
            result.expect_err("Never insert by replacement"),
            Error::Missing
        );
    }
}
