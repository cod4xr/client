// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn only_exact_successors_apply_and_later_records_compare_against_staged_sequence() {
    let mut slots = vec![ClientIdentitySlot::default(); 4];
    let mut state = ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
    seed(&mut state, 10, &[(1, b"Original", b"")]);
    let records = [
        update(12, 2, b"Gap"),
        update(11, 1, b"Next%"),
        update(11, 9, b"Duplicate"),
        update(13, 9, b"Gap"),
        update(12, 3, b"Next"),
        update(14, 9, b"Gap"),
        update(13, 2, b"Final"),
    ];
    let mut scratch = vec![ClientIdentitySlot::default(); 4];
    let before = snapshot(&state);
    let staged = state
        .prepare_updates(&records, BUDGET, &mut scratch)
        .expect("Complete batch");
    assert_eq!(
        (
            staged.summary().accepted_records,
            staged.summary().ignored_records,
            staged.summary().sequence
        ),
        (3, 4, 13)
    );
    drop(staged);
    assert_eq!(snapshot(&state), before);
    let change = state
        .prepare_updates(&records, BUDGET, &mut scratch)
        .expect("Retry")
        .commit();
    assert!(!change.baseline_replaced);
    assert_eq!(state.accepted_sequence(), Some(13));
    assert_eq!(state.generation(), 1);
    assert_eq!(
        state.clients().map(ClientData::index).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert!(state.client(9).is_none());
    assert_eq!(state.client(1).expect("Normalized").name(), b"Next.");
}

#[test]
fn exact_next_preserves_all_sequence_bits_across_signed_and_unsigned_boundaries() {
    for start in [0x7fff_fffe, 0xffff_fffe] {
        let mut slots = vec![ClientIdentitySlot::default(); 1];
        let mut state =
            ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
        seed(&mut state, start, &[(3, b"Original", b"")]);
        let records = [
            update(start.wrapping_add(1), 3, b"One"),
            update(start.wrapping_add(3), 3, b"Gap"),
            update(start.wrapping_add(2), 3, b"Two"),
            update(start.wrapping_add(3), 3, b"Three"),
            update(start, 3, b"Old"),
        ];
        let mut scratch = vec![ClientIdentitySlot::default(); 1];
        let change = state
            .prepare_updates(&records, BUDGET, &mut scratch)
            .expect("Full-width batch")
            .commit();
        assert_eq!((change.accepted_records, change.ignored_records), (3, 2));
        assert_eq!(state.accepted_sequence(), Some(start.wrapping_add(3)));
        assert_eq!(state.client(3).expect("Latest").name(), b"Three");
    }
}

#[test]
fn ignored_updates_do_not_need_new_clients_or_advance_any_sequence() {
    let mut slots = vec![ClientIdentitySlot::default(); 1];
    let mut state = ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
    seed(&mut state, 50, &[(3, b"Private", b"")]);
    let before = snapshot(&state);
    let records: Vec<_> = (0..1000).map(|_| update(52, 63, b"Ignored gap")).collect();
    let mut scratch = vec![ClientIdentitySlot::default(); 1];
    let change = state
        .prepare_updates(
            &records,
            ClientIdentityBudget {
                max_clients: 1,
                max_records: 1000,
            },
            &mut scratch,
        )
        .expect("All ignored")
        .commit();
    assert_eq!(change.ignored_records, 1000);
    assert_eq!(change.accepted_records, 0);
    assert_eq!(snapshot(&state), before);
    let _ = state
        .prepare_updates(
            &[],
            ClientIdentityBudget {
                max_clients: 1,
                max_records: 0,
            },
            &mut scratch,
        )
        .expect("Empty batch")
        .commit();
    assert_eq!(snapshot(&state), before);
}

#[test]
fn input_updates_can_be_dropped_while_staged_owned_records_remain_valid_and_private() {
    let mut slots = vec![ClientIdentitySlot::default(); 1];
    let mut state = ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
    seed(&mut state, 1, &[(3, b"Before", b"")]);
    let mut scratch = vec![ClientIdentitySlot::default(); 1];
    let staged = {
        let records = [update(2, 3, b"Private%")];
        state
            .prepare_updates(&records, BUDGET, &mut scratch)
            .expect("Owned staging")
    };
    assert!(!format!("{staged:?}").contains("Private"));
    assert_eq!(staged.clients().next().expect("Record").name(), b"Private.");
    let _ = staged.commit();
    assert!(!format!("{state:?}").contains("Private"));
}
