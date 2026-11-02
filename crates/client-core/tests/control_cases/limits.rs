// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::{MAX_COMMAND_BYTES, MessageReader, MessageStringLimit};
use cod4xr_client_core::{
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    control::{
        ClientControlHeader, ControlMessageError, ControlMessagePlan, MAX_CONTROL_MESSAGE_BYTES,
        ServerCommandKey,
    },
};

use super::{CHALLENGE, build, decoded_body, modern};

#[test]
fn retained_server_keys_validate_representation_without_claiming_history_provenance() {
    for (sequence, bytes) in [
        (0, b"not initial".as_slice()),
        (1, b"percent%"),
        (1, b"embedded\0suffix"),
        (u32::MAX, b""),
        (1, &[b'x'; MAX_COMMAND_BYTES + 1]),
    ] {
        assert_eq!(
            ServerCommandKey::from_retained(sequence, bytes).expect_err("Invalid representation"),
            ControlMessageError::ServerKey
        );
    }
    let key = ServerCommandKey::from_retained(7, b"PrivateServerKey.\x92\xff")
        .expect("Native retained bytes");
    assert_eq!(key.sequence(), 7);
    assert_eq!(key.as_bytes(), b"PrivateServerKey.\x92\xff");
    assert!(!format!("{key:?}").contains("PrivateServerKey"));
    assert!(ServerCommandKey::from_retained(1, b"").is_ok());
    assert!(ServerCommandKey::from_retained(1, &[b'x'; MAX_COMMAND_BYTES]).is_ok());
}

#[test]
fn channel_acknowledgement_is_checked_while_server_identifiers_preserve_raw_bits() {
    let mut storage = vec![0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("History");
    history.enqueue(b"PrivateCommand").expect("Pending");
    let before = format!("{history:?}");
    let header = ClientControlHeader::Cod4x21 {
        server_id: u32::MAX,
        message_acknowledge: 0x8000_0000,
        server_command: ServerCommandKey::initial(),
        config_acknowledge: u32::MAX,
    };
    assert!(matches!(
        ControlMessagePlan::new(header, CHALLENGE, &history),
        Err(ControlMessageError::Header)
    ));
    assert_eq!(format!("{history:?}"), before);
    assert_eq!(
        history.command(1).expect("Retained").as_bytes(),
        b"PrivateCommand"
    );
    let valid = ClientControlHeader::Cod4x21 {
        server_id: u32::MAX,
        message_acknowledge: 0x7fff_ffff,
        server_command: ServerCommandKey::initial(),
        config_acknowledge: u32::MAX,
    };
    let frame = ControlMessagePlan::new(valid, CHALLENGE, &history).expect("Full identifier bits");
    assert_eq!(
        &frame.as_bytes()[..16],
        &[
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff
        ]
    );
    assert_eq!(format!("{history:?}"), before);
}

#[test]
fn logical_capacity_defers_pending_prefix_without_truncating_or_marking_it_transmitted() {
    let mut storage = vec![0xa5; 4 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("History");
    for byte in [b'a', b'b', b'c'] {
        history
            .enqueue(&[byte; MAX_COMMAND_BYTES])
            .expect("Queued maximum");
    }
    let before = format!("{history:?}");
    let frame = ControlMessagePlan::new(modern(), CHALLENGE, &history).expect("Fitting prefix");
    assert_eq!(
        (frame.command_count(), frame.last_command_sequence()),
        (1, Some(1))
    );
    assert_eq!(format!("{history:?}"), before);
    assert!(frame.encoded_len() <= MAX_CONTROL_MESSAGE_BYTES);
    let body = decoded_body(&frame);
    let mut reader = MessageReader::try_new(&body).expect("Message");
    assert_eq!(reader.read_bits(3).expect("Command"), 2);
    assert_eq!(reader.read_i32().expect("Sequence"), 1);
    let mut command = [0xa5; MAX_COMMAND_BYTES];
    assert_eq!(
        reader
            .read_legacy_string(&mut command, MessageStringLimit::Normal)
            .expect("Complete string"),
        MAX_COMMAND_BYTES
    );
    assert_eq!(command, [b'a'; MAX_COMMAND_BYTES]);
    assert_eq!(reader.read_bits(3).expect("EOF"), 3);
    history
        .mark_transmitted(1)
        .expect("Host sent complete frame");
    history
        .commit_acknowledgement(1)
        .expect("Validated peer message");
    let next = ControlMessagePlan::new(modern(), CHALLENGE, &history).expect("Next pending prefix");
    assert_eq!(
        (next.command_count(), next.last_command_sequence()),
        (1, Some(2))
    );
    assert_eq!(history.pending_count(), 2);
}

#[test]
fn compression_expansion_rebuilds_a_shorter_complete_prefix_instead_of_clamping_bytes() {
    // Symbol d5 has an 11-bit code in the pinned native tree. Two such commands
    // fit the logical 2048-byte buffer but exceed the complete encoded wire budget.
    let bytes = [0xd5; 1000];
    let frame = build(modern(), &[&bytes, &bytes]);
    assert_eq!(
        (frame.command_count(), frame.last_command_sequence()),
        (1, Some(1))
    );
    assert!(frame.encoded_len() <= MAX_CONTROL_MESSAGE_BYTES);
    let body = decoded_body(&frame);
    let mut reader = MessageReader::try_new(&body).expect("Message");
    assert_eq!(reader.read_bits(3).expect("Command"), 2);
    assert_eq!(reader.read_i32().expect("Sequence"), 1);
    let mut command = [0xa5; MAX_COMMAND_BYTES];
    assert_eq!(
        reader
            .read_legacy_string(&mut command, MessageStringLimit::Normal)
            .expect("No clamping"),
        bytes.len()
    );
    assert_eq!(&command[..bytes.len()], &bytes);
    assert_eq!(reader.read_bits(3).expect("Complete EOF"), 3);
}
