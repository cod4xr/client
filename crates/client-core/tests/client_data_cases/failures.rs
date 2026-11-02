// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{carrier, identity, parse, state};
use cod4xr_client_core::{
    BaselineProtocol, client_data::ClientDataError, message_body::BodyReader,
};

#[test]
fn profile_gate_precedes_all_reads_and_invalid_player_ids_roll_back_both_cursors() {
    for sequenced in [false, true] {
        let mut reader = BodyReader::try_new(&[], 0).expect("Empty body");
        assert_eq!(
            parse(&mut reader, BaselineProtocol::Legacy6, sequenced),
            Err(ClientDataError::UnsupportedProfile)
        );
        assert_eq!(state(&reader), (0, 0, 0));
        for index in [64, 255] {
            let input = carrier(&[index], sequenced);
            let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
            assert_eq!(reader.read_bits(3), Ok(5));
            let before = state(&reader);
            assert_eq!(
                parse(&mut reader, BaselineProtocol::Cod4x21, sequenced),
                Err(ClientDataError::ClientIndex)
            );
            assert_eq!(state(&reader), before);
            assert_eq!(reader.read_bits(5), Ok(27));
        }
    }
}

#[test]
fn every_truncation_preserves_existing_byte_and_cached_bit_state() {
    for sequenced in [false, true] {
        let input = carrier(&identity(7, b"Private%Name", b"Tag%"), sequenced);
        for end in 1..input.len() {
            let mut reader = BodyReader::try_new(&input[..end], input.len()).expect("Body");
            assert_eq!(reader.read_bits(3), Ok(5));
            let before = state(&reader);
            let error = parse(&mut reader, BaselineProtocol::Cod4x21, sequenced)
                .expect_err("Incomplete record");
            assert!(matches!(error, ClientDataError::Body(_)));
            assert_eq!(state(&reader), before);
            assert_eq!(reader.read_bits(5), Ok(27));
            assert!(!format!("{error:?} {error}").contains("Private"));
            assert!(!format!("{error:?} {error}").contains("Tag"));
        }
    }
}

#[test]
fn overlong_names_and_tags_are_rejected_atomically_instead_of_silently_truncated() {
    for sequenced in [false, true] {
        for (name_length, tag_length) in [(33, 0), (0, 13), (33, 13), (1023, 0), (0, 1024)] {
            let input = carrier(
                &identity(7, &vec![b'n'; name_length], &vec![b't'; tag_length]),
                sequenced,
            );
            let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
            assert_eq!(reader.read_bits(3), Ok(5));
            let before = state(&reader);
            let error = parse(&mut reader, BaselineProtocol::Cod4x21, sequenced)
                .expect_err("Exact wire field capacities");
            assert!(matches!(error, ClientDataError::Body(_)));
            assert_eq!(state(&reader), before);
            assert_eq!(reader.read_bits(5), Ok(27));
        }
    }
}
