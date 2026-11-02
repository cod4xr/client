// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn noncanonical_compact_float_reference_rejects_before_index_or_reader_commit() {
    let original = reference(17);
    let bad = fixture(17, Some(2), |writer| {
        kind(writer, 1, 13);
        writer.write_bits(1, 7).expect("Compact X with NaN prior");
        writer.write_u8(0).expect("Compact high byte");
    });
    let mut reader = BodyReader::try_new(&bad, bad.len()).expect("Body");
    let mut stream = decoder();
    let before = position(&reader);
    assert_eq!(
        stream
            .prepare(&mut reader)
            .expect("Index")
            .decode(Some(&original), 0),
        Err(EntityDeltaError::ReferenceFloat)
    );
    assert_eq!(position(&reader), before);
    let valid = fixture(17, Some(2), |writer| {
        kind(writer, 1, 13);
        writer.write_bits(3, 2).expect("Raw X form");
        writer
            .write_i32((0x7fc0_1234_u32 ^ 2.5_f32.to_bits()).cast_signed())
            .expect("Raw XOR X");
    });
    let mut reader = BodyReader::try_new(&valid, valid.len()).expect("Retry");
    let state = present(
        stream
            .prepare(&mut reader)
            .expect("Same index")
            .decode(Some(&original), 0)
            .expect("Raw NaN reference stays inert"),
    );
    assert_eq!(state.word(W::PositionX), 2.5_f32.to_bits());
}

#[test]
fn cancelled_index_preparation_and_wrong_reference_preserve_both_owners_for_retry() {
    let original = reference(17);
    let other = reference(18);
    let bytes = fixture(17, None, |_| {});
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    let before = position(&reader);
    let mut stream = decoder();
    let pending = stream.prepare(&mut reader).expect("Prepared index");
    assert_eq!(pending.number(), 17);
    assert!(!format!("{pending:?}").contains("2143294004"));
    drop(pending);
    assert_eq!(position(&reader), before);
    assert_eq!(
        stream
            .prepare(&mut reader)
            .expect("Retry index")
            .decode(Some(&other), 0),
        Err(EntityDeltaError::ReferenceNumber)
    );
    assert_eq!(position(&reader), before);
    assert_eq!(
        present(
            stream
                .prepare(&mut reader)
                .expect("Same index again")
                .decode(Some(&original), 0)
                .expect("Commit")
        ),
        original
    );
}

#[test]
fn every_truncated_nonzero_record_preserves_cursor_cache_and_index_authority() {
    let original = reference(17);
    let bytes = fixture(17, Some(6), |writer| {
        kind(writer, 1, 1);
        writer
            .write_i32(2.0_f32.to_bits().cast_signed())
            .expect("X");
        writer
            .write_i32(3.0_f32.to_bits().cast_signed())
            .expect("Y");
        writer.write_bits(0, 1).expect("Retained union");
        writer.write_bits(0, 1).expect("Retained angle");
        writer.write_bits(1, 1).expect("Changed Z");
        writer.write_i32(0x7fc0_5678).expect("Late raw Z");
    });
    for end in 0..bytes.len() {
        let mut stream = decoder();
        let mut reader = BodyReader::try_new(&bytes[..end], bytes.len()).expect("Truncated");
        let before = position(&reader);
        let outcome = stream
            .prepare(&mut reader)
            .and_then(|pending| pending.decode(Some(&original), 0));
        assert!(outcome.is_err());
        assert_eq!(position(&reader), before);
        let mut retry = BodyReader::try_new(&bytes, bytes.len()).expect("Complete retry");
        assert_eq!(
            present(
                stream
                    .prepare(&mut retry)
                    .expect("Same index")
                    .decode(Some(&original), 0)
                    .expect("Complete fields")
            )
            .word(W::PositionZ),
            0x7fc0_5678
        );
    }
}

#[test]
fn late_count_failure_keeps_a_preexisting_partial_byte_and_previous_stream_index() {
    let first = fixture(16, None, |_| {});
    let mut stream = decoder();
    let mut first_reader = BodyReader::try_new(&first, first.len()).expect("First");
    let _ = stream
        .prepare(&mut first_reader)
        .expect("First index")
        .decode(None, 0)
        .expect("First state");
    let mut bytes = [0; 32];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    writer.write_bits(5, 3).expect("Existing cached prefix");
    writer
        .write_bits((17 << 2) | 2, 12)
        .expect("Next absolute index");
    writer.write_bits(0, 1).expect("Not removed");
    writer.write_bits(1, 1).expect("Delta");
    writer.write_bits(60, 6).expect("Beyond type-zero list");
    kind(&mut writer, 0, 0);
    writer.write_u8(7).expect("Unconsumed following opcode");
    let mut reader = BodyReader::try_new(writer.as_bytes(), writer.as_bytes().len()).expect("Body");
    assert_eq!(reader.read_bits(3), Ok(5));
    let mut expected = reader.clone();
    let before = position(&reader);
    assert_eq!(
        stream
            .prepare(&mut reader)
            .expect("Prepared next")
            .decode(None, 0),
        Err(EntityDeltaError::FieldCount)
    );
    assert_eq!(position(&reader), before);
    assert_eq!(reader.read_bits(5), expected.read_bits(5));
    let retry = fixture(17, None, |_| {});
    let mut reader = BodyReader::try_new(&retry, retry.len()).expect("Retry");
    assert_eq!(
        present(
            stream
                .prepare(&mut reader)
                .expect("No index release")
                .decode(None, 0)
                .expect("State")
        )
        .number(),
        17
    );
}

#[test]
fn profile_order_reset_and_high_index_exhaustion_are_explicit() {
    assert!(matches!(
        EntityDeltaDecoder::new(BaselineProtocol::Legacy6),
        Err(EntityDeltaError::UnsupportedProfile)
    ));
    let bytes = fixture(1023, None, |_| {});
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Last index");
    let mut stream = decoder();
    let _ = stream
        .prepare(&mut reader)
        .expect("Last")
        .decode(None, 0)
        .expect("State");
    let mut next = BodyReader::try_new(&[1], 1).expect("Increment beyond namespace");
    let before = position(&next);
    assert!(stream.prepare(&mut next).is_err());
    assert_eq!(position(&next), before);
    let mut old = BodyReader::try_new(&bytes, bytes.len()).expect("Duplicate");
    assert!(stream.prepare(&mut old).is_err());
    stream.reset();
    let _ = stream
        .prepare(&mut old)
        .expect("Explicit new stream")
        .decode(None, 0)
        .expect("State");
}
