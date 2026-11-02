// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_client_core::commands::{
    ClientCommandError, LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands,
};

#[test]
fn acknowledgement_lease_cancels_then_copies_original_key_before_pending_slot_reuse() {
    let mut bytes = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut bytes).expect("History");
    history.enqueue(b"OriginalPrivateKey").expect("Command");
    history.mark_transmitted(1).expect("Sent prefix");
    let pending = history.prepare_acknowledgement(1).expect("Prepared");
    assert_eq!(pending.sequence(), 1);
    assert!(!format!("{pending:?}").contains("OriginalPrivateKey"));
    drop(pending);
    assert_eq!(history.acknowledged_sequence(), 0);
    assert_eq!(history.pending_count(), 1);
    history.prepare_acknowledgement(1).expect("Retry").commit();
    assert_eq!(history.acknowledged_sequence(), 1);
    history
        .enqueue(b"Replacement")
        .expect("Reused pending slot");
    assert_eq!(
        history
            .command(1)
            .expect("Retained original key")
            .as_bytes(),
        b"OriginalPrivateKey"
    );
    history
        .prepare_acknowledgement(1)
        .expect("Repeated ACK")
        .commit();
    assert_eq!(
        history.command(1).expect("Repeated key").as_bytes(),
        b"OriginalPrivateKey"
    );
    assert_eq!(history.pending_count(), 1);
}
#[test]
fn zero_repeated_unsent_and_regressing_acknowledgements_preserve_existing_semantics() {
    let mut bytes = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut bytes).expect("History");
    history
        .prepare_acknowledgement(0)
        .expect("Initial key")
        .commit();
    history.enqueue(b"Key").expect("Command");
    assert_eq!(
        history.prepare_acknowledgement(1).expect_err("Unsent"),
        ClientCommandError::UnsentAcknowledgement
    );
    assert_eq!(history.acknowledged_sequence(), 0);
    history.mark_transmitted(1).expect("Sent");
    history.commit_acknowledgement(1).expect("Existing API");
    assert_eq!(
        history.prepare_acknowledgement(0).expect_err("Regression"),
        ClientCommandError::AcknowledgementRegression
    );
    assert_eq!(history.acknowledged_sequence(), 1);
    assert_eq!(
        history
            .command(1)
            .expect("Original retained key")
            .as_bytes(),
        b"Key"
    );
}
