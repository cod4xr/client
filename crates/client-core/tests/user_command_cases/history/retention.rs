// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::user_commands::zero;
use cod4xr_client_core::{
    BaselineProtocol,
    user_commands::history::{InputCommandHistory, InputCommandSlot, InputHistoryError},
};

#[test]
fn caller_capacity_exceeds_native_ring_without_aliasing_and_wraps_only_explicitly_retired_cells() {
    for profile in [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21] {
        let mut cells = vec![InputCommandSlot::default(); 257];
        let mut owner =
            InputCommandHistory::new(7, profile, &mut cells).expect("Large caller history");
        let mut ids = Vec::new();
        for time in 0..257 {
            let mut input = zero();
            input.server_time = time;
            ids.push(owner.enqueue(input).expect("No native ring ceiling"));
        }
        assert_eq!(
            (owner.len(), owner.capacity(), owner.pending_count()),
            (257, 257, 257)
        );
        for (index, &id) in ids.iter().enumerate() {
            assert_eq!(
                owner.command(id).expect("Exact retained tag").server_time,
                i32::try_from(index).expect("Bounded time")
            );
        }
        assert_eq!(owner.enqueue(zero()), Err(InputHistoryError::Capacity));
        assert_eq!(
            owner
                .id_for_local_sequence(32)
                .expect("No parallel ID queue needed"),
            ids[31]
        );
        assert_eq!(
            owner.id_for_local_sequence(0),
            Err(InputHistoryError::Missing)
        );
        let sent = owner
            .prepare_range(ids[0], ids[31])
            .expect("Native packet prefix")
            .commit_transmission();
        assert_eq!((sent.command_count, sent.newly_transmitted), (32, 32));
        assert_eq!(
            owner
                .retire_through(ids[31])
                .expect("Explicit host reconciliation"),
            32
        );
        for time in 257..289 {
            let mut input = zero();
            input.server_time = time;
            let id = owner.enqueue(input).expect("Only retired cells reusable");
            assert_eq!(
                owner.command(id).expect("New exact value").server_time,
                time
            );
        }
        assert_eq!((owner.len(), owner.pending_count()), (257, 257));
        assert_eq!(owner.command(ids[0]), Err(InputHistoryError::Missing));
        assert_eq!(
            owner.id_for_local_sequence(1),
            Err(InputHistoryError::Missing)
        );
        assert_eq!(
            owner
                .command(ids[32])
                .expect("No overwritten live input")
                .server_time,
            32
        );
        assert_eq!(
            owner.latest_id().expect("Full local counter").sequence(),
            289
        );
    }
}
#[test]
fn retirement_requires_sent_inputs_and_preserves_exact_remaining_tags_across_partial_wrap() {
    let mut cells = vec![InputCommandSlot::default(); 5];
    let mut owner =
        InputCommandHistory::new(7, BaselineProtocol::Cod4x21, &mut cells).expect("Owner");
    let ids: Vec<_> = (0..5)
        .map(|_| owner.enqueue(zero()).expect("Input"))
        .collect();
    assert_eq!(
        owner.retire_through(ids[0]),
        Err(InputHistoryError::Retirement)
    );
    let _ = owner
        .prepare_range(ids[0], ids[2])
        .expect("First send")
        .commit_transmission();
    assert_eq!(owner.retire_through(ids[1]).expect("Retire prefix"), 2);
    assert_eq!(
        owner.retire_through(ids[1]).expect("Repeated retirement"),
        0
    );
    assert_eq!(
        owner.retire_through(ids[0]),
        Err(InputHistoryError::Retirement)
    );
    assert_eq!(
        owner.retire_through(ids[3]),
        Err(InputHistoryError::Retirement)
    );
    let new_a = owner.enqueue(zero()).expect("First released cell");
    let new_b = owner.enqueue(zero()).expect("Second released cell");
    assert_eq!((owner.len(), owner.pending_count()), (5, 4));
    for &id in &ids[2..] {
        assert!(owner.command(id).is_ok());
    }
    assert!(owner.command(new_a).is_ok());
    assert!(owner.command(new_b).is_ok());
    assert_eq!(owner.command(ids[1]), Err(InputHistoryError::Missing));
}
#[test]
fn reset_and_reconstruction_invalidate_namespace_without_clearing_physical_cells() {
    let mut cells = vec![InputCommandSlot::default(); 2];
    let old = {
        let mut owner =
            InputCommandHistory::new(7, BaselineProtocol::Cod4x21, &mut cells).expect("Owner");
        owner.enqueue(zero()).expect("Retained old data")
    };
    let saved = cells.clone();
    {
        let mut owner = InputCommandHistory::new(8, BaselineProtocol::Legacy6, &mut cells)
            .expect("New namespace");
        assert!(owner.is_empty());
        assert_eq!(owner.command(old), Err(InputHistoryError::Binding));
        owner
            .reset(9, BaselineProtocol::Cod4x21)
            .expect("Logical reset");
        assert_eq!(
            (
                owner.latest_id(),
                owner.transmitted_id(),
                owner.retired_id()
            ),
            (None, None, None)
        );
    }
    assert_eq!(cells, saved);
    let mut owner =
        InputCommandHistory::new(10, BaselineProtocol::Cod4x21, &mut cells).expect("Fresh owner");
    let new = owner.enqueue(zero()).expect("Restarted counter");
    assert_eq!((old.sequence(), new.sequence()), (1, 1));
    assert_ne!(old, new);
    assert_eq!(owner.command(old), Err(InputHistoryError::Binding));
    assert!(owner.command(new).is_ok());
}
