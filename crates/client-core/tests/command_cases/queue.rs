// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::MAX_COMMAND_BYTES;
use cod4xr_client_core::commands::{
    ClientCommandError, LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands,
    MAX_LEGACY_COMMAND_STORAGE_BYTES, MAX_LEGACY_PENDING_COMMANDS,
};

use crate::snapshot;

#[test]
fn canonical_commands_own_only_initialized_bytes_and_keep_private_data_out_of_debug() {
    let mut storage = vec![0xa5; 3 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("Two pending slots");
    assert_eq!(history.capacity(), 2);
    assert_eq!(history.pending_count(), 0);
    assert_eq!(history.command(0).expect("Initial key").as_bytes(), b"");
    let mut source = b"PrivatePassword%\x92\xff".to_vec();
    assert_eq!(history.enqueue(&source), Ok(1));
    source.fill(0);
    drop(source);
    let command = history.command(1).expect("Owned command");
    assert_eq!(command.as_bytes(), b"PrivatePassword.'\xff");
    assert_eq!(command.sequence(), 1);
    for debug in [format!("{history:?}"), format!("{command:?}")] {
        assert!(!debug.contains("PrivatePassword"));
    }
    assert_eq!(history.enqueue(b""), Ok(2));
    assert_eq!(history.command(2).expect("Empty command").as_bytes(), b"");
}

#[test]
fn caller_storage_requires_complete_slots_and_preserves_dirty_bytes_on_rejection() {
    for size in [
        0,
        LEGACY_COMMAND_SLOT_BYTES,
        2 * LEGACY_COMMAND_SLOT_BYTES - 1,
        MAX_LEGACY_COMMAND_STORAGE_BYTES + 1,
        MAX_LEGACY_COMMAND_STORAGE_BYTES + LEGACY_COMMAND_SLOT_BYTES,
    ] {
        let mut storage = vec![0xa5; size];
        assert!(matches!(
            LegacyClientCommands::new(&mut storage),
            Err(ClientCommandError::Storage)
        ));
        assert!(storage.iter().all(|&byte| byte == 0xa5));
    }
    for capacity in 1..=MAX_LEGACY_PENDING_COMMANDS {
        let mut storage = vec![0xa5; (capacity + 1) * LEGACY_COMMAND_SLOT_BYTES];
        let history = LegacyClientCommands::new(&mut storage).expect("Chosen capacity");
        assert_eq!(history.capacity(), capacity);
        assert!(history.command(0).expect("Empty key").as_bytes().is_empty());
    }
}

#[test]
fn length_terminator_and_pending_capacity_failures_preserve_commands_and_sequences() {
    let mut storage = vec![0xa5; 3 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("History");
    history.enqueue(b"first").expect("First");
    let before = snapshot(&history);
    for bytes in [
        b"embedded\0PrivatePassword".as_slice(),
        &[b'x'; MAX_COMMAND_BYTES + 1],
    ] {
        assert!(history.enqueue(bytes).is_err());
        assert_eq!(snapshot(&history), before);
    }
    assert_eq!(history.enqueue(&[b'x'; MAX_COMMAND_BYTES]), Ok(2));
    assert_eq!(
        history.command(2).expect("Exact limit").as_bytes(),
        &[b'x'; MAX_COMMAND_BYTES]
    );
    let full = snapshot(&history);
    assert_eq!(
        history.enqueue(b"overflow"),
        Err(ClientCommandError::PendingCapacity)
    );
    assert_eq!(snapshot(&history), full);
    assert_eq!(
        history.command(3).expect_err("No future slot"),
        ClientCommandError::UnretainedSequence
    );
}

#[test]
fn full_128_command_window_retains_every_pending_entry_and_initial_key() {
    let mut storage = vec![0xa5; MAX_LEGACY_COMMAND_STORAGE_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("Full baseline window");
    for sequence in 1..=128 {
        assert_eq!(
            history.enqueue(format!("PrivateCommand{sequence}").as_bytes()),
            Ok(sequence)
        );
    }
    assert_eq!(history.pending_count(), 128);
    assert_eq!(
        history.command(0).expect("Separate initial key").as_bytes(),
        b""
    );
    let full = snapshot(&history);
    assert_eq!(
        history.enqueue(b"overflow"),
        Err(ClientCommandError::PendingCapacity)
    );
    assert_eq!(snapshot(&history), full);
    history
        .mark_transmitted(128)
        .expect("Admitted outgoing prefix");
    history
        .commit_acknowledgement(64)
        .expect("Validated prefix");
    for sequence in 129..=192 {
        history
            .enqueue(format!("PrivateCommand{sequence}").as_bytes())
            .expect("Reuse acknowledged slots");
    }
    assert_eq!(history.pending_count(), 128);
    for sequence in 64..=192 {
        assert_eq!(
            history
                .command(sequence)
                .expect("Key or pending")
                .as_bytes(),
            format!("PrivateCommand{sequence}").as_bytes()
        );
    }
    assert_eq!(
        history
            .command(63)
            .expect_err("Old slot cannot alias new data"),
        ClientCommandError::UnretainedSequence
    );
}
