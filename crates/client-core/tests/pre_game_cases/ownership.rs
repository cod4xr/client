// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn notices_keep_plan_bytes_without_retaining_the_history_owner_or_private_sources() {
    let mut output = [0; 128];
    let first = plan(
        &[(1, b"l Private%Map dm")],
        7,
        BaselineProtocol::Cod4x21,
        &mut output,
    );
    let mut next_output = [0; 128];
    let next = plan(
        &[(2, b"w NewReason")],
        8,
        BaselineProtocol::Cod4x21,
        &mut next_output,
    );
    let mut bytes = vec![0; SERVER_COMMAND_SLOT_BYTES];
    let mut slots = [ServerCommandSlot::default()];
    let committed = {
        let mut history =
            ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut bytes, &mut slots)
                .expect("History");
        let committed = history
            .prepare(&first)
            .expect("First")
            .with_pre_game(BUDGET)
            .expect("Typed")
            .commit();
        let _ = history.prepare(&next).expect("History available").commit();
        assert_eq!(
            history.command(2).expect("Overwritten slot").as_bytes(),
            b"w NewReason"
        );
        committed
    };
    bytes.fill(0);
    slots.fill(ServerCommandSlot::default());
    assert_eq!(
        owned(committed.events().next().expect("Original notice").action()),
        Notice::Map(b"Private.Map".to_vec(), b"dm".to_vec())
    );
}

#[test]
fn failed_outgoing_ack_preflight_cancels_typed_notices_and_privacy_is_preserved() {
    let mut output = [0; 128];
    let plan = plan(
        &[(1, b"w PrivateReason PB"), (2, b"l PrivateMap PrivateType")],
        7,
        BaselineProtocol::Cod4x21,
        &mut output,
    );
    let mut bytes = vec![0xa5; SERVER_COMMAND_SLOT_BYTES];
    let original = bytes.clone();
    let mut slots = [ServerCommandSlot::default()];
    let mut history = ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut bytes, &mut slots)
        .expect("History");
    let mut outgoing_bytes = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut outgoing = LegacyClientCommands::new(&mut outgoing_bytes).expect("Outgoing");
    let typed = history
        .prepare(&plan)
        .expect("Commands")
        .with_pre_game(BUDGET)
        .expect("Typed");
    assert!(!format!("{typed:?}").contains("Private"));
    for event in typed.events() {
        assert!(!format!("{event:?} {:?}", event.action()).contains("Private"));
    }
    assert!(outgoing.commit_acknowledgement(1).is_err());
    drop(typed);
    assert_eq!(
        (history.newest_sequence(), history.last_message_sequence()),
        (0, 0)
    );
    assert_eq!(bytes, original);
    let mut history = ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut bytes, &mut slots)
        .expect("Retry history");
    let committed = history
        .prepare(&plan)
        .expect("Commands")
        .with_pre_game(BUDGET)
        .expect("Typed")
        .commit();
    assert!(!format!("{committed:?}").contains("Private"));
    for error in [PreGameError::CommandByteBudget, PreGameError::EventBudget] {
        assert!(!format!("{error:?}: {error}").contains("Private"));
    }
}
