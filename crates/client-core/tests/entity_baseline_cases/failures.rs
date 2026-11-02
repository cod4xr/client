// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{decoder, fixture, header};
use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    gamestate::entity_baselines::{EntityBaselineDecoder, EntityBaselineError},
    message_body::BodyReader,
};

fn state(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}

#[test]
fn unsupported_profile_duplicate_order_and_explicit_stream_reset_remain_separate() {
    assert!(matches!(
        EntityBaselineDecoder::new(BaselineProtocol::Legacy6),
        Err(EntityBaselineError::UnsupportedProfile)
    ));
    let input = fixture(17, None, 0, |_| {});
    let mut decoder = decoder();
    let mut reader = BodyReader::try_new(&input, input.len()).expect("First");
    decoder.parse(&mut reader).expect("First number");
    for number in [17, 16, 0] {
        let input = fixture(number, None, 0, |_| {});
        let mut reader = BodyReader::try_new(&input, input.len()).expect("Invalid order");
        let before = state(&reader);
        assert!(decoder.parse(&mut reader).is_err());
        assert_eq!(state(&reader), before);
    }
    let next = fixture(18, None, 0, |_| {});
    let mut reader = BodyReader::try_new(&next, next.len()).expect("Next");
    assert_eq!(
        decoder
            .parse(&mut reader)
            .expect("Rejected input did not advance")
            .number(),
        18
    );
    decoder.reset();
    let mut reader = BodyReader::try_new(&input, input.len()).expect("New gamestate");
    assert_eq!(
        decoder.parse(&mut reader).expect("Explicit reset").number(),
        17
    );
}

#[test]
fn every_truncated_record_rolls_back_reader_and_index_authority_before_retry() {
    let input = fixture(17, Some(6), 1, |writer| {
        writer.write_i32(0x1234_5678).expect("Unconditional X");
        writer.write_i32(0x2345_6789).expect("Unconditional Y");
        writer.write_bits(0, 1).expect("Unchanged union");
        writer.write_bits(0, 1).expect("Unchanged angle");
        writer.write_bits(1, 1).expect("Changed Z");
        writer.write_i32(0x3456_789a).expect("Late Z bytes");
    });
    for end in 0..input.len() {
        let mut decoder = decoder();
        let mut reader = BodyReader::try_new(&input[..end], input.len()).expect("Truncated body");
        let before = state(&reader);
        assert!(decoder.parse(&mut reader).is_err());
        assert_eq!(state(&reader), before);
        let mut retry = BodyReader::try_new(&input, input.len()).expect("Complete retry");
        assert_eq!(
            decoder
                .parse(&mut retry)
                .expect("No index release on failure")
                .number(),
            17
        );
    }
}

#[test]
fn removal_and_invalid_changed_counts_preserve_the_same_index_for_a_later_valid_record() {
    let mut invalid = vec![vec![0x82, 0x10]]; // Absolute number 32, followed by removed=1.
    for (count, kind) in [(0, 0), (60, 0), (59, 12), (61, 13), (63, 255)] {
        invalid.push(fixture(32, Some(count), kind, |_| {}));
    }
    for (position, input) in invalid.iter().enumerate() {
        let mut decoder = decoder();
        let mut reader = BodyReader::try_new(input, input.len()).expect("Invalid record");
        let before = state(&reader);
        let error = decoder
            .parse(&mut reader)
            .expect_err("No malformed baseline admission");
        assert_eq!(
            error,
            if position == 0 {
                EntityBaselineError::Removed
            } else {
                EntityBaselineError::FieldCount
            }
        );
        assert_eq!(state(&reader), before);
        let valid = fixture(32, None, 0, |_| {});
        let mut retry = BodyReader::try_new(&valid, valid.len()).expect("Retry");
        assert_eq!(
            decoder.parse(&mut retry).expect("Still available").number(),
            32
        );
    }
}

#[test]
fn exhausted_increment_and_invalid_initial_short_delta_do_not_consume_cached_bits() {
    let mut decoder = decoder();
    let maximum = fixture(1023, None, 0, |_| {});
    let mut reader = BodyReader::try_new(&maximum, maximum.len()).expect("Maximum number");
    decoder.parse(&mut reader).expect("Namespace maximum");
    let mut reader = BodyReader::try_new(&[0xfd], 1).expect("Increment past maximum");
    let before = state(&reader);
    assert!(decoder.parse(&mut reader).is_err());
    assert_eq!(state(&reader), before);
    decoder.reset();
    let mut reader = BodyReader::try_new(&[0], 1).expect("Delta zero from initial minus one");
    let before = state(&reader);
    assert!(decoder.parse(&mut reader).is_err());
    assert_eq!(state(&reader), before);
    assert_eq!(reader.read_bits(6), Ok(0));
}

#[test]
fn late_field_count_failure_preserves_an_existing_partial_bit_byte_and_index_state() {
    let mut bytes = [0; 32];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Fixture");
    writer.write_bits(5, 3).expect("Existing cached field");
    header(&mut writer, 17, Some(63), 1);
    let mut reader = BodyReader::try_new(writer.as_bytes(), 32).expect("Body");
    assert_eq!(reader.read_bits(3), Ok(5));
    let before = state(&reader);
    let expected_tail = reader.clone().read_bits(5).expect("Original cache bits");
    let mut decoder = decoder();
    assert_eq!(
        decoder.parse(&mut reader),
        Err(EntityBaselineError::FieldCount)
    );
    assert_eq!(state(&reader), before);
    assert_eq!(reader.read_bits(5), Ok(expected_tail));
    let valid = fixture(17, None, 0, |_| {});
    let mut retry = BodyReader::try_new(&valid, valid.len()).expect("Retry");
    assert_eq!(
        decoder
            .parse(&mut retry)
            .expect("Index not consumed")
            .number(),
        17
    );
}
