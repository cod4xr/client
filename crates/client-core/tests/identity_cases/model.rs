// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use std::collections::BTreeMap;

#[test]
fn model_traces_cover_exact_next_wrap_duplicates_gaps_last_wins_and_cancellation() {
    for trace in 0..256_u32 {
        let mut slots = vec![ClientIdentitySlot::default(); 16];
        let mut state =
            ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
        let mut sequence = u32::MAX - 8;
        seed(&mut state, sequence, &[]);
        let mut expected = BTreeMap::new();
        let mut random = trace + 1;
        let mut scratch = vec![ClientIdentitySlot::default(); 16];
        for batch in 0..16_u32 {
            let mut records = Vec::new();
            let mut prospective = sequence;
            for index in 0..(trace + batch) % 21 {
                random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let next = match random % 4 {
                    0 => prospective.wrapping_add(2),
                    1 => prospective,
                    _ => prospective.wrapping_add(1),
                };
                if next == prospective.wrapping_add(1) {
                    prospective = next;
                }
                let name = format!("Private-{batch}-{index}%");
                records.push(update(
                    next,
                    u8::try_from((random >> 8) % 16).expect("Sparse key"),
                    name.as_bytes(),
                ));
            }
            let mut model = expected.clone();
            let mut proposed_sequence = sequence;
            let mut accepted = 0;
            for record in &records {
                if record.sequence() == proposed_sequence.wrapping_add(1) {
                    proposed_sequence = record.sequence();
                    model.insert(record.data().index(), record.data().clone());
                    accepted += 1;
                }
            }
            let before = snapshot(&state);
            let staged = state
                .prepare_updates(
                    &records,
                    ClientIdentityBudget {
                        max_clients: 16,
                        max_records: 21,
                    },
                    &mut scratch,
                )
                .expect("Bounded model batch");
            assert_eq!(staged.summary().accepted_records, accepted);
            assert_eq!(staged.summary().sequence, proposed_sequence);
            assert_eq!(staged.summary().clients, model.len());
            if (trace + batch) % 3 == 0 {
                drop(staged);
                assert_eq!(snapshot(&state), before);
            } else {
                let _ = staged.commit();
                sequence = proposed_sequence;
                expected = model;
            }
            assert_eq!(state.accepted_sequence(), Some(sequence));
            assert_eq!(state.generation(), 1);
            assert_eq!(state.len(), expected.len());
            assert_eq!(
                state.clients().cloned().collect::<Vec<_>>(),
                expected.values().cloned().collect::<Vec<_>>()
            );
        }
    }
}
