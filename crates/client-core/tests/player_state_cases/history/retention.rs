// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    PredictionHistory, PredictionHistoryError as Error, PredictionSlot, insert, prediction,
    profiles,
};

#[test]
fn retains_more_than_native_ring_without_ordering_wrapping_times_or_implicit_eviction() {
    for profile in profiles() {
        let mut slots = vec![PredictionSlot::default(); 257];
        let mut history = PredictionHistory::new(7, profile, &mut slots).expect("Owner");
        assert_eq!(
            (history.capacity(), history.len(), history.is_empty()),
            (257, 0, true)
        );
        for value in 0..257_u32 {
            let time = u32::MAX.wrapping_sub(value.wrapping_mul(7919));
            insert(&mut history, time, value);
        }
        for value in 0..257_u32 {
            let time = u32::MAX.wrapping_sub(value.wrapping_mul(7919));
            assert_eq!(
                *history.sample_at(7, time).expect("Exact"),
                prediction(profile, time, value)
            );
        }
        let sample = prediction(profile, 0, 999);
        assert_eq!(
            history
                .prepare_insert(7, sample)
                .expect_err("No implicit eviction"),
            Error::Capacity
        );
        assert_eq!(history.sample_at(7, 0), Err(Error::Missing));
        assert_eq!(history.len(), 257);
        history.remove(7, u32::MAX).expect("Explicit removal");
        history
            .prepare_insert(7, sample)
            .expect("Released capacity")
            .commit();
        assert_eq!(history.sample_at(7, 0), Ok(&sample));
        assert_eq!(history.sample_at(7, u32::MAX), Err(Error::Missing));
        assert_eq!(history.len(), 257);
    }
}

#[test]
fn insertion_and_replay_replacement_are_explicit_cancellable_and_counted_once() {
    for profile in profiles() {
        let mut slots = [PredictionSlot::default(), PredictionSlot::default()];
        let mut history = PredictionHistory::new(7, profile, &mut slots).expect("Owner");
        let original = prediction(profile, 123, 111);
        let changed = prediction(profile, 123, 999);
        let prepared = history.prepare_insert(7, original).expect("Preflight");
        assert!(!prepared.is_replacement());
        drop(prepared);
        assert!(history.is_empty());
        assert_eq!(history.sample_at(7, 123), Err(Error::Missing));
        history.prepare_insert(7, original).expect("Retry").commit();
        assert_eq!(
            history
                .prepare_insert(7, changed)
                .expect_err("Explicit replay required"),
            Error::Duplicate
        );
        let prepared = history
            .prepare_replace(7, changed)
            .expect("Replay preflight");
        assert!(prepared.is_replacement());
        drop(prepared);
        assert_eq!(history.sample_at(7, 123), Ok(&original));
        history
            .prepare_replace(7, changed)
            .expect("Replay retry")
            .commit();
        assert_eq!(history.sample_at(7, 123), Ok(&changed));
        assert_eq!(history.len(), 1);
        assert_eq!(
            history
                .prepare_replace(7, prediction(profile, 124, 0))
                .expect_err("Not insertion"),
            Error::Missing
        );
        assert_eq!(history.remove(7, 124), Err(Error::Missing));
        history.remove(7, 123).expect("Explicit release");
        assert!(history.is_empty());
    }
}

#[test]
fn every_full_small_table_removal_order_preserves_remaining_exact_keys() {
    for profile in profiles() {
        for order in permutations(vec![0, 1, 2, 3, 4]) {
            let mut slots = vec![PredictionSlot::default(); 5];
            let mut history = PredictionHistory::new(7, profile, &mut slots).expect("Owner");
            let times = [0, u32::MAX, 0x8000_0000, 0x8000_0001, 0x0101_0101];
            for (value, &time) in (0_u32..).zip(&times) {
                insert(&mut history, time, value);
            }
            let mut retained = [true; 5];
            for index in order {
                history.remove(7, times[index]).expect("Cluster deletion");
                retained[index] = false;
                for (value, (&time, &present)) in (0_u32..).zip(times.iter().zip(&retained)) {
                    assert_eq!(
                        history.sample_at(7, time).copied(),
                        present
                            .then(|| prediction(profile, time, value))
                            .ok_or(Error::Missing)
                    );
                }
            }
            assert!(history.is_empty());
        }
    }
}
fn permutations(mut values: Vec<usize>) -> Vec<Vec<usize>> {
    if values.is_empty() {
        return vec![vec![]];
    }
    let last = values.pop().expect("Nonempty");
    permutations(values)
        .into_iter()
        .flat_map(|prefix| {
            (0..=prefix.len()).map(move |index| {
                let mut order = prefix.clone();
                order.insert(index, last);
                order
            })
        })
        .collect()
}
