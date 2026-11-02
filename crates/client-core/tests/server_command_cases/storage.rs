// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::MAX_COMMAND_BYTES;
use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    server_commands::{
        SERVER_COMMAND_SLOT_BYTES, ServerCommandError, ServerCommandHistory, ServerCommandSlot,
        required_server_command_bytes,
    },
};

use super::{Fixture, append_command, plan};

#[test]
fn storage_geometry_is_checked_before_mutation_and_dirty_data_is_initially_opaque() {
    assert_eq!(
        required_server_command_bytes(0),
        Err(ServerCommandError::Storage)
    );
    assert_eq!(
        required_server_command_bytes(usize::MAX),
        Err(ServerCommandError::Storage)
    );
    for (count, bytes) in [
        (0, 0),
        (1, SERVER_COMMAND_SLOT_BYTES - 1),
        (1, SERVER_COMMAND_SLOT_BYTES + 1),
        (2, SERVER_COMMAND_SLOT_BYTES),
    ] {
        let mut slots = vec![ServerCommandSlot::default(); count];
        let original = slots.clone();
        let mut data = vec![0xa5; bytes];
        assert!(matches!(
            ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut data, &mut slots),
            Err(ServerCommandError::Storage)
        ));
        assert_eq!(slots, original);
        assert!(data.iter().all(|&byte| byte == 0xa5));
    }
    for count in [1, 3, 128, 256] {
        let mut slots = vec![ServerCommandSlot::default(); count];
        let mut data = vec![0xa5; required_server_command_bytes(count).expect("Checked geometry")];
        {
            let history =
                ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut data, &mut slots)
                    .expect("Budgeted history");
            assert_eq!(history.capacity(), count);
            assert_eq!(history.profile(), BaselineProtocol::Cod4x21);
            assert_eq!(
                (history.newest_sequence(), history.last_message_sequence()),
                (0, 0)
            );
            assert!(
                history
                    .latest_key()
                    .expect("Initial key")
                    .as_bytes()
                    .is_empty()
            );
            for sequence in [0, 1, u32::MAX] {
                assert!(matches!(
                    history.command(sequence),
                    Err(ServerCommandError::Missing)
                ));
            }
        }
        assert!(data.iter().all(|&byte| byte == 0xa5));
    }
}

#[test]
fn prepared_cancellation_preserves_bytes_metadata_and_sequence_state() {
    let mut client_data = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let client = LegacyClientCommands::new(&mut client_data).expect("Client history");
    let mut source = Vec::new();
    append_command(&mut source, 1, b"PrivateCancelled");
    source.push(7);
    let mut fixture = Fixture::new(&source, 0, b"");
    let mut command_data = [0; 64];
    let plan = plan(&mut fixture, &client, &mut command_data);
    let mut data = vec![0xa5; SERVER_COMMAND_SLOT_BYTES];
    let mut slots = [ServerCommandSlot::default()];
    {
        let mut history =
            ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut data, &mut slots)
                .expect("History");
        {
            let prepared = history.prepare(&plan).expect("Prepared");
            assert_eq!(prepared.summary().new_commands, 1);
            assert!(!format!("{prepared:?}").contains("PrivateCancelled"));
        }
        assert_eq!(
            (history.newest_sequence(), history.last_message_sequence()),
            (0, 0)
        );
        assert!(history.command(1).is_err());
    }
    assert!(data.iter().all(|&byte| byte == 0xa5));
    assert_eq!(slots, [ServerCommandSlot::default()]);
}

#[test]
fn varied_non_power_of_two_capacities_tag_reused_slots_and_retain_more_than_native_128() {
    let mut client_data = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let client = LegacyClientCommands::new(&mut client_data).expect("Client history");
    for capacity in [1_usize, 2, 3, 128, 256] {
        let mut data = vec![0xa5; required_server_command_bytes(capacity).expect("Geometry")];
        let mut slots = vec![ServerCommandSlot::default(); capacity];
        let mut history =
            ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut data, &mut slots)
                .expect("History");
        let latest = i32::try_from(2 * capacity + 1).expect("Fixture sequence");
        let mut first = 1;
        let mut message = 7;
        while first <= latest {
            let end = (first + 127).min(latest);
            let mut source = Vec::new();
            for sequence in first..=end {
                append_command(
                    &mut source,
                    sequence,
                    format!("command-{sequence}").as_bytes(),
                );
            }
            source.push(7);
            let mut fixture = Fixture::at_sequence(&source, 0, b"", message);
            let mut output = [0; 4096];
            let plan = plan(&mut fixture, &client, &mut output);
            let result = history.prepare(&plan).expect("Prepared").commit();
            assert_eq!(
                result.newest_command,
                u32::try_from(end).expect("Last sequence")
            );
            first = end + 1;
            message += 1;
        }
        let latest = u32::try_from(latest).expect("Latest");
        let oldest = latest - u32::try_from(capacity).expect("Capacity") + 1;
        for sequence in oldest..=latest {
            assert_eq!(
                history
                    .command(sequence)
                    .expect("Retained exact tag")
                    .as_bytes(),
                format!("command-{sequence}").as_bytes()
            );
        }
        assert_eq!(
            history.command(oldest - 1).expect_err("Reused old slot"),
            ServerCommandError::Missing
        );
        if capacity > 128 {
            assert_eq!(
                history
                    .command(latest - 200)
                    .expect("History beyond native 128")
                    .as_bytes(),
                format!("command-{}", latest - 200).as_bytes()
            );
        }
    }
}

#[test]
fn full_big_command_survives_retention_and_shorter_reuse_exposes_no_stale_tail() {
    let mut client_data = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let client = LegacyClientCommands::new(&mut client_data).expect("Client history");
    let mut data = vec![0xa5; SERVER_COMMAND_SLOT_BYTES];
    let mut slots = [ServerCommandSlot::default()];
    let mut history = ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut data, &mut slots)
        .expect("History");
    let mut text = vec![b'x'; SERVER_COMMAND_SLOT_BYTES - 1];
    text[0] = 0x92;
    text[MAX_COMMAND_BYTES - 1] = b'%';
    let mut source = Vec::new();
    append_command(&mut source, 1, &text);
    source.push(7);
    let mut fixture = Fixture::new(&source, 0, b"");
    let mut output = vec![0; text.len()];
    let plan = plan(&mut fixture, &client, &mut output);
    let _summary = history.prepare(&plan).expect("Prepared").commit();
    assert_eq!(
        history.command(1).expect("Full command").as_bytes().len(),
        text.len()
    );
    assert_eq!(
        history.latest_key().expect("Key prefix").as_bytes().len(),
        MAX_COMMAND_BYTES
    );
    assert_eq!(
        history
            .latest_key()
            .expect("Native byte preservation")
            .as_bytes()[0],
        0x92
    );
    assert_eq!(
        history.latest_key().expect("Normalized key").as_bytes()[MAX_COMMAND_BYTES - 1],
        b'.'
    );
    let mut empty = Fixture::at_sequence(b"\x04\x02\0\0\0\0\x07", 0, b"", 8);
    let mut output = [];
    let plan = super::plan(&mut empty, &client, &mut output);
    let _summary = history.prepare(&plan).expect("Short reuse").commit();
    assert!(history.command(1).is_err());
    assert!(
        history
            .command(2)
            .expect("New empty command")
            .as_bytes()
            .is_empty()
    );
    assert!(
        history
            .latest_key()
            .expect("No old tail")
            .as_bytes()
            .is_empty()
    );
}
