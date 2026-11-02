// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn baseline_input_distinct_and_private_capacities_reject_without_owner_changes() {
    let mut slots = vec![ClientIdentitySlot::default(); 3];
    let mut state = ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
    seed(&mut state, 90, &[(3, b"Private", b"")]);
    let before = snapshot(&state);
    let mut source = vec![ClientDataSlot::default(); 3];
    let plan = baseline(
        4,
        2,
        &[(2, b"First", b""), (1, b"Second", b""), (2, b"Last", b"")],
        &mut source,
    );
    let mut scratch = vec![ClientIdentitySlot::default(); 2];
    for (budget, length, error) in [
        (
            ClientIdentityBudget {
                max_records: 2,
                ..BUDGET
            },
            2,
            ClientIdentityError::RecordBudget,
        ),
        (
            ClientIdentityBudget {
                max_clients: 1,
                ..BUDGET
            },
            2,
            ClientIdentityError::ClientBudget,
        ),
        (BUDGET, 1, ClientIdentityError::ScratchCapacity),
    ] {
        assert_eq!(
            state
                .prepare_game_state(&plan, budget, &mut scratch[..length])
                .err(),
            Some(error)
        );
        assert_eq!(snapshot(&state), before);
    }
    let change = state
        .prepare_game_state(
            &plan,
            ClientIdentityBudget {
                max_records: 3,
                max_clients: 2,
            },
            &mut scratch,
        )
        .expect("Exact independent budgets")
        .commit();
    assert_eq!((change.records, change.clients), (3, 2));
}

#[test]
fn late_baseline_output_failure_preserves_every_final_cell_and_generation() {
    let mut slots = vec![ClientIdentitySlot::default(); 1];
    {
        let mut state =
            ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
        seed(&mut state, 8, &[(3, b"Private", b"")]);
    }
    let before = slots.clone();
    {
        let mut state =
            ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("New owner");
        let mut source = vec![ClientDataSlot::default(); 2];
        let plan = baseline(4, 1, &[(1, b"One", b""), (2, b"Two", b"")], &mut source);
        let mut scratch = vec![ClientIdentitySlot::default(); 2];
        assert_eq!(
            state.prepare_game_state(&plan, BUDGET, &mut scratch).err(),
            Some(ClientIdentityError::OutputCapacity)
        );
        assert_eq!(state.generation(), 0);
        assert_eq!(state.accepted_sequence(), None);
    }
    assert_eq!(slots, before);
}

#[test]
fn late_update_capacity_failure_rolls_back_all_prior_proposed_records_and_sequences() {
    for failure in 0..3 {
        let mut slots = vec![ClientIdentitySlot::default(); 3];
        let mut state =
            ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
        seed(
            &mut state,
            9,
            &[(1, b"Private", b""), (3, b"Original", b"")],
        );
        let before = snapshot(&state);
        let records = [
            update(10, 1, b"Proposed replacement"),
            update(11, 2, b"Proposed insert"),
            update(12, 4, b"Late overflow"),
        ];
        let mut scratch = vec![ClientIdentitySlot::default(); 4];
        let (budget, length, error) = match failure {
            0 => (BUDGET, 4, ClientIdentityError::OutputCapacity),
            1 => (BUDGET, 2, ClientIdentityError::ScratchCapacity),
            _ => (
                ClientIdentityBudget {
                    max_clients: 2,
                    ..BUDGET
                },
                4,
                ClientIdentityError::ClientBudget,
            ),
        };
        assert_eq!(
            state
                .prepare_updates(&records, budget, &mut scratch[..length])
                .err(),
            Some(error)
        );
        assert_eq!(snapshot(&state), before);
        let _ = state
            .prepare_updates(&records[..1], BUDGET, &mut scratch)
            .expect("Dirty workspace can retry")
            .commit();
        assert_eq!(state.accepted_sequence(), Some(10));
        assert_eq!(
            state.client(1).expect("Committed retry").name(),
            b"Proposed replacement"
        );
        assert!(state.client(2).is_none());
    }
}

#[test]
fn update_inspection_and_retention_budgets_include_ignored_and_existing_records() {
    let mut slots = vec![ClientIdentitySlot::default(); 2];
    let mut state = ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
    seed(
        &mut state,
        9,
        &[(1, b"Private", b""), (3, b"Original", b"")],
    );
    let before = snapshot(&state);
    let mut scratch = vec![ClientIdentitySlot::default(); 2];
    assert_eq!(
        state
            .prepare_updates(
                &[update(30, 1, b"Ignored")],
                ClientIdentityBudget {
                    max_records: 0,
                    ..BUDGET
                },
                &mut scratch
            )
            .err(),
        Some(ClientIdentityError::RecordBudget)
    );
    assert_eq!(
        state
            .prepare_updates(
                &[],
                ClientIdentityBudget {
                    max_clients: 1,
                    ..BUDGET
                },
                &mut scratch
            )
            .err(),
        Some(ClientIdentityError::ClientBudget)
    );
    assert_eq!(
        state.prepare_updates(&[], BUDGET, &mut scratch[..1]).err(),
        Some(ClientIdentityError::ScratchCapacity)
    );
    assert_eq!(snapshot(&state), before);
}

#[test]
fn cancelled_batches_and_unused_final_tails_remain_unmodified() {
    let mut slots = vec![ClientIdentitySlot::default(); 4];
    {
        let mut state =
            ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
        seed(&mut state, 9, &[(1, b"Private", b"")]);
        let before = snapshot(&state);
        let mut scratch = vec![ClientIdentitySlot::default(); 4];
        let staged = state
            .prepare_updates(&[update(10, 63, b"Proposed")], BUDGET, &mut scratch)
            .expect("Staged");
        drop(staged);
        assert_eq!(snapshot(&state), before);
        let _ = state
            .prepare_updates(&[update(10, 1, b"Committed")], BUDGET, &mut scratch)
            .expect("One replacement")
            .commit();
        assert_eq!(state.len(), 1);
    }
    assert_eq!(slots[1..], vec![ClientIdentitySlot::default(); 3]);
}
