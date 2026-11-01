// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::{compression, obfuscation};
use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    server_payload::ServerPayloadDecoder,
};

use crate::{CHALLENGE, INITIAL, KEYED, decoder};

#[test]
fn profiles_map_only_explicit_reviewed_connect_numbers_without_hint_fallback() {
    for profile in [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21] {
        assert_eq!(
            BaselineProtocol::from_number(profile.number()),
            Some(profile)
        );
    }
    for number in [i32::MIN, -1, 0, 18, 22, i32::MAX] {
        assert_eq!(BaselineProtocol::from_number(number), None);
    }
}

#[test]
fn both_baseline_server_profiles_match_independent_initial_key_vectors_and_preserve_padding() {
    for profile in [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21] {
        let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let history = LegacyClientCommands::new(&mut storage).expect("Initial history");
        let decoder = ServerPayloadDecoder::new(profile, CHALLENGE);
        let mut work = [0xa5; 8];
        let mut output = [0xa5; 16];
        {
            let message = decoder
                .decode(7, &INITIAL, &history, &mut work, &mut output)
                .expect("Native vector");
            assert_eq!(message.profile(), profile);
            assert_eq!(message.sequence(), 7);
            assert_eq!(message.client_command_acknowledge(), 0);
            // Two extra symbols follow EOF due to native byte-framed zero padding.
            assert_eq!(message.body(), &[0, 7, 6, 6]);
            assert!(!format!("{message:?}").contains("305419896"));
        }
        assert_eq!(&output[4..], &[0xa5; 12]);
        assert_eq!(&work[2..], &[0xa5; 6]);
        assert_eq!(history.acknowledged_sequence(), 0);
        assert!(!format!("{decoder:?}").contains("305419896"));
    }
}

#[test]
fn exact_transmitted_canonical_command_selects_the_key_without_committing_acknowledgement() {
    let mut storage = [0xa5; 3 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("History");
    history.enqueue(b"a%").expect("Canonical a.");
    history
        .enqueue(b"PrivateUnsentCommand")
        .expect("Second command");
    history.mark_transmitted(1).expect("First command sent");
    let before = format!("{history:?}");
    let mut work = [0xa5; 8];
    let mut output = [0xa5; 16];
    let source = KEYED;
    let number = {
        let message = decoder()
            .decode(7, &source, &history, &mut work, &mut output)
            .expect("Retained key");
        assert_eq!(message.body(), &[0, 7, 6, 6]);
        assert_eq!(message.client_command_acknowledge(), 1);
        assert_eq!(format!("{history:?}"), before);
        message.client_command_acknowledge()
    };
    assert_eq!(source, KEYED);
    assert_eq!(history.acknowledged_sequence(), 0);
    // The future service layer validates its complete stream before this transition.
    history
        .commit_acknowledgement(number)
        .expect("Explicit post-validation commit");
    assert_eq!(history.pending_count(), 1);
    assert_eq!(history.command(1).expect("Retained key").as_bytes(), b"a.");
}

#[test]
fn opaque_decoded_bytes_do_not_become_validated_commands_or_debug_payloads() {
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut storage).expect("History");
    // Fixture construction is used only for privacy/opacity, not wire conformance.
    let source = b"\xfePrivatePeerCommand\0";
    let mut packet = [0; 128];
    let length = compression::compress(source, &mut packet[4..]).expect("Fixture compression");
    obfuscation::apply_server_payload(&mut packet[4..4 + length], 0x1234_5678, 7, b"")
        .expect("Fixture transform");
    let mut work = [0xa5; 128];
    let mut output = [0xa5; 128];
    let message = decoder()
        .decode(7, &packet[..4 + length], &history, &mut work, &mut output)
        .expect("Opaque bytes");
    assert!(message.body().starts_with(source));
    assert!(!format!("{message:?}").contains("PrivatePeerCommand"));
    assert_eq!(history.acknowledged_sequence(), 0);
}
