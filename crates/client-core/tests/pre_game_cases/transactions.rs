// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn fresh_notices_follow_wire_order_even_when_retained_slots_collide() {
    let mut output = [0; 128];
    let plan = plan(
        &[
            (3, b"l map dm"),
            (1, b"w stale"),
            (3, b"w duplicate"),
            (5, b"m"),
            (6, b"w reason PB"),
        ],
        7,
        BaselineProtocol::Cod4x21,
        &mut output,
    );
    let mut bytes = vec![0xa5; SERVER_COMMAND_SLOT_BYTES];
    let mut slots = [ServerCommandSlot::default()];
    let mut history = ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut bytes, &mut slots)
        .expect("History");
    let pending = history.prepare(&plan).expect("Commands");
    assert_eq!(
        pending
            .new_commands()
            .map(|command| command.sequence())
            .collect::<Vec<_>>(),
        [3, 5, 6]
    );
    let typed = pending.with_pre_game(BUDGET).expect("Typed");
    assert_eq!(
        typed
            .events()
            .map(cod4xr_client_core::pre_game::PreGameEvent::sequence)
            .collect::<Vec<_>>(),
        [3, 5, 6]
    );
    assert_eq!(
        (
            typed.summary().new_commands,
            typed.summary().ignored_commands
        ),
        (3, 2)
    );
    let committed = typed.commit();
    assert_eq!(
        committed
            .events()
            .map(cod4xr_client_core::pre_game::PreGameEvent::sequence)
            .collect::<Vec<_>>(),
        [3, 5, 6]
    );
    assert_eq!(history.newest_sequence(), 6);
    assert!(history.command(3).is_err());
    assert_eq!(
        history
            .command(6)
            .expect("Last collision wins retention")
            .as_bytes(),
        b"w reason PB"
    );
}

#[test]
fn byte_and_event_failures_cancel_the_consumed_history_preparation_with_all_bytes_unchanged() {
    let mut output = [0; 128];
    let plan = plan(
        &[(1, b"l map dm"), (2, b"unknown Private"), (3, b"m")],
        7,
        BaselineProtocol::Cod4x21,
        &mut output,
    );
    let mut bytes = vec![0xa5; 2 * SERVER_COMMAND_SLOT_BYTES];
    let original = bytes.clone();
    let mut slots = [ServerCommandSlot::default(); 2];
    {
        let mut history =
            ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut bytes, &mut slots)
                .expect("History");
        for (budget, expected) in [
            (
                PreGameBudget {
                    max_command_bytes: 23,
                    max_events: 2,
                },
                PreGameError::CommandByteBudget,
            ),
            (
                PreGameBudget {
                    max_command_bytes: 24,
                    max_events: 1,
                },
                PreGameError::EventBudget,
            ),
            (
                PreGameBudget {
                    max_command_bytes: usize::MAX,
                    max_events: 0,
                },
                PreGameError::EventBudget,
            ),
        ] {
            assert_eq!(
                history
                    .prepare(&plan)
                    .expect("Retry")
                    .with_pre_game(budget)
                    .err(),
                Some(expected)
            );
            assert_eq!(
                (history.newest_sequence(), history.last_message_sequence()),
                (0, 0)
            );
        }
        let pending = history
            .prepare(&plan)
            .expect("Retry")
            .with_pre_game(PreGameBudget {
                max_command_bytes: 24,
                max_events: 2,
            })
            .expect("Exact budgets");
        drop(pending);
        assert_eq!(
            (history.newest_sequence(), history.last_message_sequence()),
            (0, 0)
        );
    }
    assert_eq!(bytes, original);
    assert!(
        slots
            .iter()
            .all(|slot| *slot == ServerCommandSlot::default())
    );
}

#[test]
fn ignored_records_cost_no_new_command_work_and_unknown_commands_still_retain_their_key() {
    let mut first_output = [0; 128];
    let first = plan(
        &[(3, b"m")],
        7,
        BaselineProtocol::Cod4x21,
        &mut first_output,
    );
    let mut next_output = [0; 128];
    let next = plan(
        &[(1, b"l old dm"), (3, b"w duplicate"), (4, b"unknown%key")],
        8,
        BaselineProtocol::Cod4x21,
        &mut next_output,
    );
    let mut bytes = vec![0; SERVER_COMMAND_SLOT_BYTES];
    let mut slots = [ServerCommandSlot::default()];
    let mut history = ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut bytes, &mut slots)
        .expect("History");
    let _ = history.prepare(&first).expect("First").commit();
    let committed = history
        .prepare(&next)
        .expect("Next")
        .with_pre_game(PreGameBudget {
            max_command_bytes: 11,
            max_events: 0,
        })
        .expect("Only new unknown text charged")
        .commit();
    assert_eq!(committed.events().count(), 0);
    assert_eq!(
        (
            committed.summary().new_commands,
            committed.summary().ignored_commands
        ),
        (1, 2)
    );
    assert_eq!(
        history
            .latest_key()
            .expect("Unknown still affects next frame")
            .as_bytes(),
        b"unknown.key"
    );
}

#[test]
fn empty_new_commands_and_all_duplicate_messages_can_use_zero_event_and_byte_budgets() {
    let mut output = [0; 64];
    let first = plan(&[(1, b"")], 7, BaselineProtocol::Cod4x21, &mut output);
    let mut next_output = [0; 64];
    let next = plan(
        &[(1, b"w ignored")],
        8,
        BaselineProtocol::Cod4x21,
        &mut next_output,
    );
    let mut bytes = vec![0; SERVER_COMMAND_SLOT_BYTES];
    let mut slots = [ServerCommandSlot::default()];
    let mut history = ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut bytes, &mut slots)
        .expect("History");
    for (plan, count) in [(&first, 1), (&next, 0)] {
        let committed = history
            .prepare(plan)
            .expect("Whole plan")
            .with_pre_game(PreGameBudget {
                max_command_bytes: 0,
                max_events: 0,
            })
            .expect("No inspection/publication work")
            .commit();
        assert_eq!(committed.summary().new_commands, count);
        assert_eq!(committed.events().count(), 0);
    }
    assert_eq!(history.last_message_sequence(), 8);
    assert!(
        history
            .command(1)
            .expect("Original empty record")
            .as_bytes()
            .is_empty()
    );
}
