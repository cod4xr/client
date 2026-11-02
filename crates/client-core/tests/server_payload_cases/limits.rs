// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::{MAX_MESSAGE_BYTES, compression::CompressionError};
use cod4xr_client_core::{
    commands::{ClientCommandError, LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    server_payload::ServerPayloadError,
};

use super::{INITIAL, KEYED, decoder};

#[test]
fn unsent_future_negative_and_regressing_acknowledgements_reject_before_buffer_writes() {
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("History");
    history
        .enqueue(b"PrivateClientKey")
        .expect("Queued but unsent");
    let mut work = [0xa5; 8];
    let mut output = [0xa5; 16];
    for number in [1_u32, 2, u32::MAX] {
        let mut packet = KEYED;
        packet[..4].copy_from_slice(&number.to_le_bytes());
        assert!(matches!(
            decoder().decode(7, &packet, &history, &mut work, &mut output),
            Err(ServerPayloadError::Acknowledgement(
                ClientCommandError::UnsentAcknowledgement
            ))
        ));
        assert_eq!(work, [0xa5; 8]);
        assert_eq!(output, [0xa5; 16]);
    }
    history.mark_transmitted(1).expect("Sent");
    history
        .commit_acknowledgement(1)
        .expect("Previously validated");
    assert!(matches!(
        decoder().decode(7, &INITIAL, &history, &mut work, &mut output),
        Err(ServerPayloadError::Acknowledgement(
            ClientCommandError::AcknowledgementRegression
        ))
    ));
    assert_eq!(work, [0xa5; 8]);
    assert_eq!(output, [0xa5; 16]);
    assert_eq!(
        history.command(1).expect("Retained private key").as_bytes(),
        b"PrivateClientKey"
    );
}

#[test]
fn every_short_scratch_or_decoded_output_fails_atomically_at_its_public_boundary() {
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut storage).expect("History");
    for capacity in 0..2 {
        let mut work = [0xa5; 8];
        let mut output = [0xa5; 16];
        assert!(matches!(
            decoder().decode(7, &INITIAL, &history, &mut work[..capacity], &mut output),
            Err(ServerPayloadError::WorkCapacity)
        ));
        assert_eq!(work, [0xa5; 8]);
        assert_eq!(output, [0xa5; 16]);
    }
    for capacity in 0..4 {
        let mut work = [0xa5; 8];
        let mut output = [0xa5; 16];
        assert!(matches!(
            decoder().decode(7, &INITIAL, &history, &mut work, &mut output[..capacity]),
            Err(ServerPayloadError::Compression(
                CompressionError::OutputCapacity
            ))
        ));
        assert_eq!(output, [0xa5; 16]);
        assert_eq!(history.acknowledged_sequence(), 0);
    }
}

#[test]
fn incomplete_empty_oversized_input_and_invalid_sequences_do_not_mutate_output() {
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut storage).expect("History");
    let mut work = [0xa5; 8];
    let mut output = [0xa5; 16];
    for length in 0..4 {
        assert!(matches!(
            decoder().decode(7, &INITIAL[..length], &history, &mut work, &mut output),
            Err(ServerPayloadError::Header)
        ));
    }
    assert!(matches!(
        decoder().decode(7, &INITIAL[..4], &history, &mut work, &mut output),
        Err(ServerPayloadError::Empty)
    ));
    for sequence in [0, 0x8000_0000, u32::MAX] {
        assert!(matches!(
            decoder().decode(sequence, &INITIAL, &history, &mut work, &mut output),
            Err(ServerPayloadError::Sequence)
        ));
    }
    let mut oversized = vec![0; MAX_MESSAGE_BYTES + 1];
    assert!(matches!(
        decoder().decode(7, &oversized, &history, &mut work, &mut output),
        Err(ServerPayloadError::InputCapacity)
    ));
    oversized.truncate(MAX_MESSAGE_BYTES);
    assert!(matches!(
        decoder().decode(7, &oversized, &history, &mut work, &mut output),
        Err(ServerPayloadError::WorkCapacity)
    ));
    assert_eq!(work, [0xa5; 8]);
    assert_eq!(output, [0xa5; 16]);
}

#[test]
fn reserved_huffman_symbol_fails_without_publishing_output_or_committing_history() {
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut storage).expect("History");
    // Reserved symbol 256: LSB-first native code 445/11 bits, XOR seed 7f.
    let packet = [0, 0, 0, 0, 0xc2, 0x7e];
    let mut work = [0xa5; 8];
    let mut output = [0xa5; 16];
    assert!(matches!(
        decoder().decode(7, &packet, &history, &mut work, &mut output),
        Err(ServerPayloadError::Compression(
            CompressionError::ReservedSymbol
        ))
    ));
    assert_eq!(output, [0xa5; 16]);
    assert_eq!(history.acknowledged_sequence(), 0);
}
