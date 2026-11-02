// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BUDGET, Output, client, configstrings, encoded, entity, finish, mixed};
use cod4xr_client_core::{
    BaselineProtocol,
    gamestate::{GameStateBudget, GameStateError, GameStatePlan},
    message_body::BodyReader,
};

pub(super) fn rejected(
    input: &[u8],
    budget: GameStateBudget,
    scratch: usize,
    mut output: Output,
) -> GameStateError {
    let mut reader = BodyReader::try_new(input, input.len()).expect("Body");
    let before = output.clone();
    let error = GameStatePlan::parse(
        &mut reader,
        BaselineProtocol::Cod4x21,
        budget,
        &mut vec![0; scratch],
        output.storage(),
    )
    .expect_err("Complete validation required");
    assert_eq!(reader.byte_position(), 0);
    assert_eq!(reader.remaining_cached_bits(), 0);
    assert_eq!(reader.remaining_bytes(), input.len());
    assert_eq!(output, before);
    error
}

#[test]
fn every_input_truncation_preserves_all_dirty_storage_and_the_original_reader() {
    let input = encoded(|writer| mixed(writer, 63));
    for end in 0..input.len() {
        rejected(&input[..end], BUDGET, 5, Output::dirty(7, 3, 2, 2));
    }
}

#[test]
fn unknown_records_bad_trailers_and_late_nested_errors_publish_no_earlier_records() {
    for opcode in [0, 1, 4, 5, 6, 10, 12, 255] {
        let input = encoded(|writer| {
            writer.write_i32(0).expect("Header");
            configstrings(writer, &[(0, b"earlier")]);
            entity(writer, 17);
            client(writer, 63, b"private", b"tag");
            writer.write_u8(opcode).expect("Invalid inner opcode");
        });
        assert_eq!(
            rejected(&input, BUDGET, 7, Output::dirty(64, 8, 8, 8)),
            GameStateError::UnsupportedService(opcode)
        );
    }
    for own_client in [-1, 64, i32::MAX] {
        let input = encoded(|writer| mixed(writer, own_client));
        assert_eq!(
            rejected(&input, BUDGET, 5, Output::dirty(7, 3, 2, 2)),
            GameStateError::ClientIndex
        );
    }
    for bad in 0..3 {
        let input = encoded(|writer| {
            writer.write_i32(0).expect("Header");
            configstrings(writer, &[(0, b"earlier")]);
            entity(writer, 17);
            client(writer, 63, b"private", b"tag");
            match bad {
                0 => configstrings(writer, &[(17, b"one"), (17, b"duplicate")]),
                1 => entity(writer, 17),
                _ => client(writer, 64, b"invalid", b""),
            }
            finish(writer, 0);
        });
        let error = rejected(&input, BUDGET, 16, Output::dirty(64, 8, 8, 8));
        assert!(matches!(
            error,
            GameStateError::ConfigStrings(_)
                | GameStateError::Entity(_)
                | GameStateError::Client(_)
        ));
    }
}

#[test]
fn unsupported_profile_precedes_reads_and_late_failures_keep_existing_cache_contents() {
    let mut output = Output::dirty(7, 3, 2, 2);
    let before = output.clone();
    let mut reader = BodyReader::try_new(&[], 0).expect("Empty body");
    assert_eq!(
        GameStatePlan::parse(
            &mut reader,
            BaselineProtocol::Legacy6,
            BUDGET,
            &mut [],
            output.storage()
        )
        .expect_err("Explicit profile"),
        GameStateError::UnsupportedProfile
    );
    assert_eq!(reader.byte_position(), 0);
    assert_eq!(output, before);
    let input = encoded(|writer| {
        writer.write_bits(5, 3).expect("Existing cache");
        mixed(writer, 64);
    });
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
    assert_eq!(reader.read_bits(3), Ok(5));
    let before_position = reader.byte_position();
    let expected = reader.clone().read_bits(5).expect("Original tail");
    assert_eq!(
        GameStatePlan::parse(
            &mut reader,
            BaselineProtocol::Cod4x21,
            BUDGET,
            &mut [0; 5],
            output.storage()
        )
        .expect_err("Late trailer failure"),
        GameStateError::ClientIndex
    );
    assert_eq!(reader.byte_position(), before_position);
    assert_eq!(reader.remaining_cached_bits(), 5);
    assert_eq!(reader.read_bits(5), Ok(expected));
    assert_eq!(output, before);
}
