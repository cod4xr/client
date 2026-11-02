// SPDX-License-Identifier: AGPL-3.0-or-later
use super::support::full;
use crate::snapshots::{SnapshotClientSlot, SnapshotEntitySlot, contexts};
use cod4xr_client_core::snapshots::history::{SnapshotHistory, SnapshotHistorySlot};
use std::collections::VecDeque;

#[test]
fn circular_exact_lookup_matches_an_independent_queue_across_gaps_cancellation_and_resets() {
    for context in contexts() {
        for capacity in [1, 2, 3, 7, 40] {
            let mut entity_cells = vec![vec![SnapshotEntitySlot::default(); 1]; capacity];
            let mut client_cells = vec![vec![SnapshotClientSlot::default(); 1]; capacity];
            let mut slots: Vec<_> = entity_cells
                .iter_mut()
                .zip(&mut client_cells)
                .map(|(entities, clients)| SnapshotHistorySlot::new(entities, clients))
                .collect();
            let mut expected = VecDeque::new();
            let mut epoch = 7;
            {
                let mut history = SnapshotHistory::new(epoch, context.profile(), &mut slots)
                    .expect("Host storage");
                let mut entity_output = [SnapshotEntitySlot::default()];
                let mut client_output = [SnapshotClientSlot::default()];
                let mut random = 0x9135_7231_u32;
                let mut sequence = 0;
                for step in 0..256 {
                    if step != 0 && step % 53 == 0 {
                        epoch += 1;
                        history.reset(epoch, context.profile()).expect("New world");
                        expected.clear();
                        sequence = 0;
                    }
                    random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    sequence += 1 + random % 7;
                    let plan = full(
                        context,
                        epoch,
                        sequence,
                        &mut entity_output,
                        &mut client_output,
                    );
                    let prepared = history.prepare(&plan).expect("All storage preflight");
                    if random.is_multiple_of(8) {
                        drop(prepared);
                    } else {
                        let evicted = if expected.len() == capacity {
                            expected.pop_front()
                        } else {
                            None
                        };
                        expected.push_back(sequence);
                        let retained = prepared.commit();
                        assert_eq!(
                            (retained.sequence, retained.evicted, retained.retained),
                            (sequence, evicted, expected.len())
                        );
                    }
                    assert_eq!(history.len(), expected.len());
                    assert_eq!(history.newest_sequence(), expected.back().copied());
                    for &key in &expected {
                        let source = history
                            .source(key)
                            .expect("Exact queue key survives physical wrap");
                        assert_eq!(
                            (source.message().epoch(), source.message().sequence()),
                            (epoch, key)
                        );
                        assert_eq!(source.profile(), context.profile());
                        assert!(history.header(key).is_some());
                    }
                    let probe = random % (sequence + 2);
                    assert_eq!(history.source(probe).is_some(), expected.contains(&probe));
                    assert!(history.source(sequence + 1).is_none());
                }
            }
            // Reconstructing over reused slots never grants old metadata validity.
            let history =
                SnapshotHistory::new(epoch + 1, context.profile(), &mut slots).expect("New owner");
            assert!(history.is_empty());
            assert_eq!(history.newest_sequence(), None);
            for key in expected {
                assert!(history.source(key).is_none());
                assert!(history.header(key).is_none());
            }
        }
    }
}
