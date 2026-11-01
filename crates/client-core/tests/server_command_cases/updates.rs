// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_client_core::{
    BaselineProtocol,
    commands::{ClientCommandError, LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    server_commands::{
        SERVER_COMMAND_SLOT_BYTES, ServerCommandError, ServerCommandHistory, ServerCommandSlot,
    },
    service_controls::{LegacyServicePlan, ServiceControlBudget},
};

use crate::{Fixture, append_command, apply_pair, plan};

#[test]
fn monotonic_handling_ignores_old_records_counts_gaps_and_keeps_private_commands_out_of_debug() {
    let mut client_data = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let client = LegacyClientCommands::new(&mut client_data).expect("Client history");
    let mut source = Vec::new();
    for (sequence, bytes) in [
        (3, b"PrivateStored%".as_slice()),
        (1, b"old"),
        (3, b"duplicate"),
        (4, b""),
    ] {
        append_command(&mut source, sequence, bytes);
    }
    source.push(7);
    let mut fixture = Fixture::new(&source, 0, b"");
    let mut output = [0; 128];
    let plan = plan(&mut fixture, &client, &mut output);
    let mut data = vec![0xa5; 8 * SERVER_COMMAND_SLOT_BYTES];
    let mut slots = [ServerCommandSlot::default(); 8];
    let mut history = ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut data, &mut slots)
        .expect("History");
    let proposed = {
        let prepared = history.prepare(&plan).expect("Prepared");
        let proposed = prepared.summary();
        assert_eq!(
            (
                proposed.new_commands,
                proposed.ignored_commands,
                proposed.skipped_sequences
            ),
            (2, 2, 2)
        );
        assert_eq!(prepared.commit(), proposed);
        proposed
    };
    assert_eq!(
        (history.newest_sequence(), history.last_message_sequence()),
        (4, 7)
    );
    assert_eq!(proposed.newest_command, 4);
    assert_eq!(
        history
            .command(3)
            .expect("First newer record wins")
            .as_bytes(),
        b"PrivateStored."
    );
    assert!(history.command(1).is_err());
    assert!(history.command(5).is_err());
    assert_eq!(history.latest_key().expect("Current key").sequence(), 4);
    assert!(
        history
            .latest_key()
            .expect("Empty key")
            .as_bytes()
            .is_empty()
    );
    for debug in [
        format!("{history:?}"),
        format!("{:?}", history.command(3).expect("Command")),
    ] {
        assert!(!debug.contains("PrivateStored"));
    }
}

#[test]
fn profile_and_stale_message_failures_leave_existing_records_unchanged() {
    let mut client_data = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let client = LegacyClientCommands::new(&mut client_data).expect("Client history");
    let source = b"\x04\x01\0\0\0PrivateRetained\0\x07";
    let mut fixture = Fixture::new(source, 0, b"");
    let mut output = [0; 64];
    let plan = plan(&mut fixture, &client, &mut output);
    let mut data = vec![0xa5; SERVER_COMMAND_SLOT_BYTES];
    let mut slots = [ServerCommandSlot::default()];
    {
        let mut different =
            ServerCommandHistory::new(BaselineProtocol::Legacy6, &mut data, &mut slots)
                .expect("Other profile");
        assert!(matches!(
            different.prepare(&plan),
            Err(ServerCommandError::Profile)
        ));
        assert_eq!(different.newest_sequence(), 0);
    }
    assert!(data.iter().all(|&byte| byte == 0xa5));
    let mut history = ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut data, &mut slots)
        .expect("Original profile");
    let _summary = history.prepare(&plan).expect("First message").commit();
    let before = format!("{history:?}");
    assert!(matches!(
        history.prepare(&plan),
        Err(ServerCommandError::MessageOrder)
    ));
    let mut old = Fixture::at_sequence(source, 0, b"", 6);
    let mut old_output = [0; 64];
    let old_plan = crate::plan(&mut old, &client, &mut old_output);
    assert!(matches!(
        history.prepare(&old_plan),
        Err(ServerCommandError::MessageOrder)
    ));
    let mut other = Fixture::at_sequence(source, 0, b"", 8);
    let other_payload = other.decode_profile(&client, BaselineProtocol::Legacy6);
    let mut other_output = [0; 64];
    let other_plan = LegacyServicePlan::parse(
        &other_payload,
        ServiceControlBudget::default(),
        &mut [0; 64],
        &mut other_output,
    )
    .expect("Other reviewed profile");
    assert!(matches!(
        history.prepare(&other_plan),
        Err(ServerCommandError::Profile)
    ));
    assert_eq!(format!("{history:?}"), before);
    assert_eq!(
        history.command(1).expect("No replacement").as_bytes(),
        b"PrivateRetained"
    );
}

#[test]
fn failing_outgoing_acknowledgement_cancels_the_prepared_incoming_update() {
    let mut client_data = [0; 3 * LEGACY_COMMAND_SLOT_BYTES];
    let mut client = LegacyClientCommands::new(&mut client_data).expect("Client history");
    client.enqueue(b"a%").expect("First key");
    client.enqueue(b"two").expect("Second key");
    client.mark_transmitted(2).expect("Both admitted to I/O");
    let source = b"\x04\x01\0\0\0PrivateIncoming%\0\x07";
    let mut fixture = Fixture::new(source, 1, b"a.");
    let mut output = [0; 64];
    let stale_plan = plan(&mut fixture, &client, &mut output);
    client
        .commit_acknowledgement(2)
        .expect("A newer validated message already advanced outgoing history");
    let mut data = vec![0xa5; SERVER_COMMAND_SLOT_BYTES];
    let mut slots = [ServerCommandSlot::default()];
    let mut history = ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut data, &mut slots)
        .expect("Server history");
    assert_eq!(
        apply_pair(&mut history, &mut client, &stale_plan),
        Err(ClientCommandError::AcknowledgementRegression)
    );
    assert_eq!(
        (history.newest_sequence(), history.last_message_sequence()),
        (0, 0)
    );
    assert!(history.command(1).is_err());
    let mut current = Fixture::at_sequence(source, 2, b"two", 8);
    let mut current_output = [0; 64];
    let current_plan = plan(&mut current, &client, &mut current_output);
    let summary = apply_pair(&mut history, &mut client, &current_plan)
        .expect("Both preflighted histories commit");
    assert_eq!((summary.message_sequence, summary.new_commands), (8, 1));
    assert_eq!(
        history
            .command(1)
            .expect("Normalized incoming data")
            .as_bytes(),
        b"PrivateIncoming."
    );
    assert_eq!(client.pending_count(), 0);
}

#[test]
fn complete_nop_message_advances_validated_message_order_without_changing_the_latest_key() {
    let mut client_data = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let client = LegacyClientCommands::new(&mut client_data).expect("Client history");
    let mut fixture = Fixture::new(b"\x04\x01\0\0\0PrivateKey\0\x07", 0, b"");
    let mut output = [0; 64];
    let first = plan(&mut fixture, &client, &mut output);
    let mut data = vec![0xa5; SERVER_COMMAND_SLOT_BYTES];
    let mut slots = [ServerCommandSlot::default()];
    let mut history = ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut data, &mut slots)
        .expect("History");
    let _summary = history.prepare(&first).expect("First message").commit();
    let mut nop = Fixture::at_sequence(&[0, 7], 0, b"", 8);
    let mut output = [];
    let next = plan(&mut nop, &client, &mut output);
    let summary = history
        .prepare(&next)
        .expect("Complete NOP message")
        .commit();
    assert_eq!(
        (
            summary.new_commands,
            summary.ignored_commands,
            summary.skipped_sequences
        ),
        (0, 0, 0)
    );
    assert_eq!(
        (history.newest_sequence(), history.last_message_sequence()),
        (1, 8)
    );
    assert_eq!(
        history
            .latest_key()
            .expect("Retained current key")
            .as_bytes(),
        b"PrivateKey"
    );
}
