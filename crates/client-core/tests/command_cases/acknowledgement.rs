// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_client_core::commands::{
    ClientCommandError, LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands,
};

use crate::snapshot;

#[test]
fn checking_acknowledgement_is_read_only_until_complete_message_validation_commits_it() {
    let mut storage = vec![0xa5; 3 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("History");
    history.enqueue(b"PrivateKey%\x92").expect("First");
    history.enqueue(b"second").expect("Second");
    assert_eq!(
        history
            .check_acknowledgement(1)
            .expect_err("Never transmitted"),
        ClientCommandError::UnsentAcknowledgement
    );
    history.mark_transmitted(2).expect("Host admitted both");
    let before = snapshot(&history);
    for _ in 0..2 {
        let checked = history.check_acknowledgement(1).expect("Inspect key");
        assert_eq!(checked.sequence(), 1);
        assert_eq!(checked.command().as_bytes(), b"PrivateKey.'");
        assert!(!format!("{checked:?}").contains("PrivateKey"));
        // A malformed decoded message abandons this view without releasing any slot.
        assert_eq!(snapshot(&history), before);
    }
    assert_eq!(
        history.enqueue(b"must still be full"),
        Err(ClientCommandError::PendingCapacity)
    );
    history
        .commit_acknowledgement(1)
        .expect("Complete validated message");
    history
        .enqueue(b"third reuses first ring slot")
        .expect("One released slot");
    assert_eq!(
        history
            .command(1)
            .expect("Separate key survives ring reuse")
            .as_bytes(),
        b"PrivateKey.'"
    );
    assert_eq!(
        history
            .command(2)
            .expect("Unacknowledged command")
            .as_bytes(),
        b"second"
    );
    assert_eq!(
        history.command(3).expect("New command").as_bytes(),
        b"third reuses first ring slot"
    );
}

#[test]
fn acknowledgements_and_transmit_prefixes_reject_future_or_backwards_references_atomically() {
    let mut storage = vec![0xa5; 4 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("History");
    for bytes in [b"one".as_slice(), b"two", b"three"] {
        history.enqueue(bytes).expect("Queued");
    }
    history.mark_transmitted(2).expect("Only first two sent");
    history
        .commit_acknowledgement(1)
        .expect("First acknowledged");
    let before = snapshot(&history);
    for sequence in [3, 4, u32::MAX] {
        assert_eq!(
            history
                .check_acknowledgement(sequence)
                .expect_err("Future or unsent"),
            ClientCommandError::UnsentAcknowledgement
        );
        assert_eq!(
            history.commit_acknowledgement(sequence),
            Err(ClientCommandError::UnsentAcknowledgement)
        );
        assert_eq!(snapshot(&history), before);
    }
    assert_eq!(
        history.commit_acknowledgement(0),
        Err(ClientCommandError::AcknowledgementRegression)
    );
    for sequence in [0, 1, 4, u32::MAX] {
        assert_eq!(
            history.mark_transmitted(sequence),
            Err(ClientCommandError::TransmitRange)
        );
        assert_eq!(snapshot(&history), before);
    }
    history
        .commit_acknowledgement(1)
        .expect("Repeated valid acknowledgement");
    assert_eq!(snapshot(&history), before);
    history.mark_transmitted(3).expect("Third sent");
    history.commit_acknowledgement(3).expect("Third validated");
    assert_eq!(history.pending_count(), 0);
    assert_eq!(
        history
            .command(3)
            .expect("Retained acknowledged key")
            .as_bytes(),
        b"three"
    );
    assert_eq!(
        history
            .command(2)
            .expect_err("Cannot alias reused/old slot"),
        ClientCommandError::UnretainedSequence
    );
}

#[test]
fn repeated_window_reuse_preserves_last_key_and_all_pending_bytes_at_every_capacity() {
    for capacity in 1..=128 {
        let mut storage = vec![0xa5; (capacity + 1) * LEGACY_COMMAND_SLOT_BYTES];
        let mut history = LegacyClientCommands::new(&mut storage).expect("Chosen window");
        let mut latest = 0;
        for _ in 0..3 {
            for _ in 0..capacity {
                latest += 1;
                assert_eq!(
                    history.enqueue(format!("command-{latest}").as_bytes()),
                    Ok(latest)
                );
            }
            let acknowledged = history.acknowledged_sequence();
            let expected_key = if acknowledged == 0 {
                Vec::new()
            } else {
                format!("command-{acknowledged}").into_bytes()
            };
            assert_eq!(
                history
                    .command(acknowledged)
                    .expect("Retained key")
                    .as_bytes(),
                expected_key
            );
            for sequence in acknowledged + 1..=latest {
                assert_eq!(
                    history
                        .command(sequence)
                        .expect("Pending sequence")
                        .as_bytes(),
                    format!("command-{sequence}").as_bytes()
                );
            }
            history
                .mark_transmitted(latest)
                .expect("All outgoing commands admitted");
            history
                .commit_acknowledgement(latest)
                .expect("Complete message validated");
            assert_eq!(history.pending_count(), 0);
        }
    }
}
