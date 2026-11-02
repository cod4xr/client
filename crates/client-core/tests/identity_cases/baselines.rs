// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn construction_never_invents_a_baseline_or_uses_dirty_caller_cells() {
    let mut slots = vec![ClientIdentitySlot::default(); 2];
    {
        let mut previous =
            ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
        seed(&mut previous, 42, &[(3, b"Private", b"Old")]);
    }
    let before = slots.clone();
    assert_eq!(
        ClientIdentityState::new(BaselineProtocol::Legacy6, &mut slots).err(),
        Some(ClientIdentityError::Profile)
    );
    assert_eq!(slots, before);
    let mut state =
        ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Fresh owner");
    assert!(state.is_empty());
    assert_eq!(state.accepted_sequence(), None);
    assert_eq!(state.own_client_index(), None);
    assert_eq!(state.generation(), 0);
    assert!(state.client(3).is_none());
    assert_eq!(
        state
            .prepare_updates(&[update(0, 3, b"No implicit sequence")], BUDGET, &mut [])
            .err(),
        Some(ClientIdentityError::MissingBaseline)
    );
}

#[test]
fn complete_baseline_last_duplicates_win_and_staging_outlives_all_source_storage() {
    let mut slots = vec![ClientIdentitySlot::default(); 3];
    let mut state = ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
    let mut scratch = vec![ClientIdentitySlot::default(); 2];
    let staged = {
        let mut input_slots = vec![ClientDataSlot::default(); 3];
        let plan = baseline(
            0xffff_fffe,
            63,
            &[
                (63, b"Private", b"Old"),
                (1, b"First", b""),
                (63, b"Last%", b"Raw\xff"),
            ],
            &mut input_slots,
        );
        state
            .prepare_game_state(&plan, BUDGET, &mut scratch)
            .expect("Complete replacement")
    };
    assert_eq!((staged.summary().records, staged.summary().clients), (3, 2));
    assert_eq!(
        staged.clients().map(ClientData::index).collect::<Vec<_>>(),
        [1, 63]
    );
    let change = staged.commit();
    assert_eq!(change.accepted_records, 3);
    assert!(change.baseline_replaced);
    assert_eq!(state.generation(), 1);
    assert_eq!(state.accepted_sequence(), Some(0xffff_fffe));
    assert_eq!(state.own_client_index(), Some(63));
    assert_eq!(state.client(63).expect("Latest duplicate").name(), b"Last.");
    assert_eq!(state.client(63).expect("Raw tag").clan_tag(), b"Raw\xff");
    assert!(state.client(u32::MAX).is_none());
    assert_eq!(state.len(), 2);
}

#[test]
fn cancellation_and_replacement_keep_generation_records_and_sequence_atomic() {
    let mut slots = vec![ClientIdentitySlot::default(); 4];
    let mut state = ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
    seed(&mut state, 100, &[(1, b"Private", b""), (63, b"Old", b"")]);
    let before = snapshot(&state);
    let mut source = vec![ClientDataSlot::default(); 1];
    let plan = baseline(2, 4, &[(4, b"Replacement", b"")], &mut source);
    let mut scratch = vec![ClientIdentitySlot::default(); 1];
    let staged = state
        .prepare_game_state(&plan, BUDGET, &mut scratch)
        .expect("Replacement");
    assert_eq!(staged.summary().generation, 2);
    drop(staged);
    assert_eq!(snapshot(&state), before);
    let _ = state
        .prepare_game_state(&plan, BUDGET, &mut scratch)
        .expect("Retry")
        .commit();
    assert_eq!(state.generation(), 2);
    assert_eq!(state.accepted_sequence(), Some(2));
    assert_eq!(state.own_client_index(), Some(4));
    assert!(state.client(1).is_none());
    assert!(state.client(63).is_none());
    assert_eq!(
        state.client(4).expect("Fresh record").name(),
        b"Replacement"
    );
}

#[test]
fn empty_baseline_is_explicit_and_does_not_require_a_global_player_arena() {
    let mut slots = [];
    let mut state = ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots)
        .expect("Zero-capacity owner");
    let plan = baseline(u32::MAX, 0, &[], &mut []);
    let _ = state
        .prepare_game_state(
            &plan,
            ClientIdentityBudget {
                max_clients: 0,
                max_records: 0,
            },
            &mut [],
        )
        .expect("Empty explicit baseline")
        .commit();
    assert!(state.is_empty());
    assert_eq!(state.accepted_sequence(), Some(u32::MAX));
    assert_eq!(state.own_client_index(), Some(0));
    assert_eq!(
        state
            .prepare_updates(
                &[update(0, 0, b"Needs capacity")],
                BUDGET,
                &mut [ClientIdentitySlot::default()]
            )
            .err(),
        Some(ClientIdentityError::OutputCapacity)
    );
    assert_eq!(state.accepted_sequence(), Some(u32::MAX));
}
