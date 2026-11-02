// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, LivePlayerState, PlayerStateError, contexts, decode, empty, encoded, mixed,
    position,
};
use cod4xr_client_core::player_hud::HudError;

#[test]
fn every_truncated_mixed_state_rolls_back_all_sections_and_cached_reader_state() {
    for context in contexts() {
        let prior = decode(
            context,
            &encoded(|writer| mixed(writer, true, 0, 100)),
            None,
        );
        let saved = prior.clone();
        let bytes = encoded(|writer| {
            writer.write_bits(5, 3).expect("Cached leading bits");
            mixed(writer, true, 0, 200);
        });
        for length in 1..bytes.len() {
            let mut reader = BodyReader::try_new(&bytes[..length], length).expect("Truncated body");
            assert_eq!(reader.read_bits(3).expect("Cached prefix"), 5);
            let before = position(&reader);
            assert!(
                LivePlayerState::prepare(&mut reader, context, 0, Some(&prior)).is_err(),
                "prefix {length}"
            );
            assert_eq!(position(&reader), before);
            assert_eq!(reader.read_bits(5), {
                let mut original = BodyReader::try_new(&bytes[..length], length).expect("Original");
                original.read_bits(3).expect("Prefix");
                original.read_bits(5)
            });
            assert_eq!(prior, saved);
        }
    }
}
#[test]
fn malformed_last_current_hud_field_cancels_valid_earlier_sections_and_archival_array() {
    for context in contexts() {
        let prior = decode(
            context,
            &encoded(|writer| mixed(writer, true, 0, 100)),
            None,
        );
        let saved = prior.clone();
        let bytes = encoded(|writer| {
            writer.write_bits(1, 1).expect("Mode");
            writer.write_bits(0, 8).expect("Scalar prefix");
            writer
                .write_bits(0, 10)
                .expect("Stats, stored, clip groups");
            writer.write_bits(0, 1).expect("Objectives unchanged");
            writer.write_bits(1, 1).expect("HUD changed");
            writer.write_bits(1, 5).expect("Archival element");
            writer.write_bits(0, 6).expect("Color prefix");
            writer.write_bits(3, 2).expect("Changed color shortcut");
            writer.write_bits(1, 5).expect("Current element");
            writer.write_bits(63, 6).expect("Invalid inclusive field");
        });
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let before = position(&reader);
        assert_eq!(
            LivePlayerState::prepare(&mut reader, context, 0, Some(&prior)).err(),
            Some(PlayerStateError::Hud(HudError::FieldCount))
        );
        assert_eq!(position(&reader), before);
        assert_eq!(prior, saved);
    }
}
#[test]
fn profile_mismatch_precedes_body_reads_and_preserves_reference() {
    let contexts = contexts();
    for (index, &context) in contexts.iter().enumerate() {
        let prior = decode(contexts[1 - index], &[1, 0, 0], None);
        let mut reader = BodyReader::try_new(&[], 0).expect("Empty body");
        let before = position(&reader);
        assert_eq!(
            LivePlayerState::prepare(&mut reader, context, 0, Some(&prior)).err(),
            Some(PlayerStateError::ReferenceProfile)
        );
        assert_eq!(position(&reader), before);
    }
}
#[test]
fn dropping_prepared_full_state_or_prediction_request_cancels_and_allows_retry() {
    for context in contexts() {
        for transmitted in [false, true] {
            let bytes = encoded(|writer| {
                writer.write_bits(3, 2).expect("Cached prefix");
                empty(writer, transmitted);
            });
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
            reader.read_bits(2).expect("Prefix");
            let before = position(&reader);
            let prepared =
                LivePlayerState::prepare(&mut reader, context, 0, None).expect("Prepared");
            assert_eq!(prepared.prediction_request().is_some(), !transmitted);
            drop(prepared);
            assert_eq!(position(&reader), before);
            let retry = LivePlayerState::prepare(&mut reader, context, 0, None).expect("Retry");
            if transmitted {
                retry.commit_transmitted().expect("Commit");
            } else {
                retry.commit_native_prior_fallback().expect("Fallback");
            }
        }
    }
}
