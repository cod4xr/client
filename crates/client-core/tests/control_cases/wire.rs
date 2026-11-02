// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::{MessageReader, MessageStringLimit};
use cod4xr_client_core::{
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    control::{ClientControlHeader, ControlMessageError, ControlMessagePlan, ServerCommandKey},
};

use super::{CHALLENGE, build, decoded_body, legacy, modern};

#[test]
fn idle_profiles_match_independent_plaintext_huffman_and_xor_vectors() {
    // EOF is the three-bit value 3: plaintext byte 03, native Huffman code 2b.
    // XOR seeds are 6e (legacy) and 39 (modern), producing 45 and 12 respectively.
    let legacy_frame = build(legacy(), &[]);
    assert_eq!(
        legacy_frame.as_bytes(),
        &[0x12, 4, 3, 2, 1, 0, 0, 0, 0, 0x45]
    );
    assert_eq!(legacy_frame.requested_protocol(), 6);
    assert_eq!(legacy().encoded_len(), 9);
    let modern_frame = build(modern(), &[]);
    assert_eq!(
        modern_frame.as_bytes(),
        &[
            0x44, 0x33, 0x22, 0x11, 5, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0x12
        ]
    );
    assert_eq!(modern_frame.requested_protocol(), 21);
    assert_eq!(modern().encoded_len(), 16);
    assert_eq!(modern_frame.command_count(), 0);
    assert_eq!(modern_frame.last_command_sequence(), None);
    assert_eq!(modern_frame.server_challenge(), CHALLENGE);
}

#[test]
fn canonical_commands_and_scattered_opcode_bits_match_independent_keyed_vectors() {
    // One command has payload 1a 01000000 612e00: command opcode + EOF share 1a.
    let legacy_frame = build(legacy(), &[b"a%"]);
    assert_eq!(
        legacy_frame.as_bytes(),
        &[
            0x12, 4, 3, 2, 1, 0, 0, 0, 0, 0x99, 0x4b, 0x67, 0xb8, 0xdc, 0x6e
        ]
    );
    // Two commands share d2; EOF's final zero bit allocates another byte after strings.
    // Expected bytes use native static Huffman tables and independently evolved XOR keys.
    let frame = {
        let mut key_bytes = b"PrivateServerKey".to_vec();
        let header = ClientControlHeader::Cod4x21 {
            server_id: 0x1122_3344,
            message_acknowledge: 5,
            server_command: ServerCommandKey::from_retained(7, &key_bytes)
                .expect("Retained server key"),
            config_acknowledge: 9,
        };
        let result = build(header, &[b"a%", b"b"]);
        assert!(!format!("{header:?}").contains("PrivateServerKey"));
        key_bytes.fill(0);
        result
    };
    assert_eq!(
        frame.as_bytes(),
        &[
            0x44, 0x33, 0x22, 0x11, 5, 0, 0, 0, 7, 0, 0, 0, 9, 0, 0, 0, 0xdc, 0xc7, 0xf6, 0xa4,
            0x0c, 0x58, 0x76, 0x5e, 0x6a, 0xc3
        ]
    );
    assert_eq!(frame.command_count(), 2);
    assert_eq!(frame.last_command_sequence(), Some(2));
    for private in ["PrivateServerKey", "305419896", "12345678"] {
        assert!(!format!("{frame:?}").contains(private));
    }
}

#[test]
fn every_short_output_is_unchanged_and_exact_output_preserves_unused_suffix() {
    let frame = build(modern(), &[b"PrivateCommand"]);
    for capacity in 0..frame.encoded_len() {
        let mut output = vec![0xa5; frame.encoded_len() + 3];
        assert_eq!(
            frame.encode(&mut output[..capacity]),
            Err(ControlMessageError::OutputCapacity)
        );
        assert!(output.iter().all(|&byte| byte == 0xa5));
    }
    let mut output = vec![0xa5; frame.encoded_len() + 3];
    assert_eq!(frame.encode(&mut output), Ok(frame.encoded_len()));
    assert_eq!(&output[..frame.encoded_len()], frame.as_bytes());
    assert_eq!(&output[frame.encoded_len()..], &[0xa5; 3]);
}

#[test]
fn full_empty_command_window_serializes_every_sequence_and_explicit_eof() {
    let mut storage = vec![0xa5; 129 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("Full baseline history");
    for _ in 0..128 {
        history.enqueue(b"").expect("Empty command");
    }
    let frame = ControlMessagePlan::new(modern(), CHALLENGE, &history).expect("Frame");
    assert_eq!(
        (frame.command_count(), frame.last_command_sequence()),
        (128, Some(128))
    );
    let body = decoded_body(&frame);
    let mut reader = MessageReader::try_new(&body).expect("Message");
    for sequence in 1..=128 {
        assert_eq!(reader.read_bits(3).expect("Command opcode"), 2);
        assert_eq!(reader.read_i32().expect("Sequence"), sequence);
        assert_eq!(
            reader
                .read_legacy_string(&mut [], MessageStringLimit::Normal)
                .expect("String"),
            0
        );
    }
    assert_eq!(reader.read_bits(3).expect("EOF"), 3);
    assert_eq!(history.transmitted_sequence(), 0);
    assert_eq!(history.pending_count(), 128);
}
