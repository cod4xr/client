// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "A valid near-exhaustion state avoids billions of setup commands."
)]
use super::{ClientCommandError, LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands};

#[test]
fn last_positive_signed_sequence_succeeds_then_reconnect_is_required_without_mutation() {
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("One pending slot");
    let last = u32::try_from(i32::MAX).expect("Positive signed bound");
    // Model a long-lived connection whose previous acknowledged command was empty.
    history.latest = last - 1;
    history.transmitted = last - 1;
    history.acknowledged = last - 1;
    assert_eq!(history.enqueue(b"PrivateFinalCommand"), Ok(last));
    history.mark_transmitted(last).expect("Last command sent");
    history
        .commit_acknowledgement(last)
        .expect("Last command validated");
    let before = format!("{history:?}");
    assert_eq!(
        history.enqueue(b"must not wrap"),
        Err(ClientCommandError::SequenceExhausted)
    );
    assert_eq!(format!("{history:?}"), before);
    assert_eq!(history.pending_count(), 0);
    assert_eq!(
        history
            .command(last)
            .expect("Retained final key")
            .as_bytes(),
        b"PrivateFinalCommand"
    );
    assert_eq!(
        history.command(0).expect_err("No wrapped initial key"),
        ClientCommandError::UnretainedSequence
    );
}
