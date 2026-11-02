// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, ClientStateDecoder, ClientStateError, PROFILES, W, WORDS, encoded, header, integer,
    position, present, raw_float, reference,
};

#[test]
fn every_truncated_full_record_preserves_cached_bits_and_index_for_retry() {
    let prior = reference(7, 0xa5a5_a5a5);
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Cached prefix");
        header(writer, 7, Some(24));
        for &(word, width) in &WORDS {
            if width == 0 {
                raw_float(writer, prior.word(word), 1.25_f32.to_bits());
            } else {
                integer(writer, width, prior.word(word), u32::MAX >> (32 - width));
            }
        }
    });
    for profile in PROFILES {
        for length in 1..bytes.len() {
            let mut owner = ClientStateDecoder::new(profile);
            let mut reader = BodyReader::try_new(&bytes[..length], length).expect("Prefix");
            reader.read_bits(3).expect("Cached bits");
            let before = position(&reader);
            assert!(
                owner
                    .prepare(&mut reader)
                    .and_then(|p| p.expect("Record").decode(Some(&prior)))
                    .is_err()
            );
            assert_eq!(position(&reader), before, "prefix {length}");
            let mut complete = BodyReader::try_new(&bytes, bytes.len()).expect("Complete body");
            complete.read_bits(3).expect("Cache");
            let state = present(
                owner
                    .prepare(&mut complete)
                    .expect("Retry header")
                    .expect("Record")
                    .decode(Some(&prior))
                    .expect("Retry"),
            );
            assert_eq!(state.word(W::SprintTimeMultiplier), 1.25_f32.to_bits());
        }
    }
}

#[test]
fn invalid_counts_and_reference_mismatch_cancel_the_whole_record() {
    for count in 25..32 {
        let bytes = encoded(|writer| header(writer, 5, Some(count)));
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let before = position(&reader);
        let mut owner = ClientStateDecoder::new(PROFILES[1]);
        assert_eq!(
            owner
                .prepare(&mut reader)
                .expect("Header")
                .expect("Record")
                .decode(None),
            Err(ClientStateError::FieldCount)
        );
        assert_eq!(position(&reader), before);
        let valid = [0x15, 0];
        let mut retry = BodyReader::try_new(&valid, 2).expect("Retry body");
        assert!(
            owner
                .prepare(&mut retry)
                .expect("Retry header")
                .expect("Record")
                .decode(None)
                .is_ok()
        );
    }
    let prior = reference(6, 0);
    let mut owner = ClientStateDecoder::new(PROFILES[1]);
    let mut reader = BodyReader::try_new(&[0x15, 0], 2).expect("Wrong reference");
    let before = position(&reader);
    assert_eq!(
        owner
            .prepare(&mut reader)
            .expect("Header")
            .expect("Record")
            .decode(Some(&prior)),
        Err(ClientStateError::ReferenceNumber)
    );
    assert_eq!(position(&reader), before);
    assert_eq!(
        present(
            owner
                .prepare(&mut reader)
                .expect("Retry header")
                .expect("Record")
                .decode(None)
                .expect("Retry")
        )
        .number(),
        5
    );
}

#[test]
fn compact_float_invalid_prior_rejects_late_without_changing_either_owner() {
    for bits in [
        0x7fc0_1234,
        f32::INFINITY.to_bits(),
        4096.0_f32.to_bits(),
        (-4097.0_f32).to_bits(),
    ] {
        let prior = reference(4, bits);
        let bytes = encoded(|writer| {
            header(writer, 4, Some(13));
            integer(writer, 9, prior.word(W::Model), 17);
            for _ in 1..12 {
                writer.write_bits(0, 1).expect("Unchanged");
            }
            writer
                .write_bits(3, 3)
                .expect("Changed nonzero compact float");
            writer.write_bits(0, 5).expect("Compact residual");
            writer.write_u8(0).expect("Compact byte");
        });
        let mut owner = ClientStateDecoder::new(PROFILES[1]);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let before = position(&reader);
        assert_eq!(
            owner
                .prepare(&mut reader)
                .expect("Header")
                .expect("Record")
                .decode(Some(&prior)),
            Err(ClientStateError::ReferenceFloat)
        );
        assert_eq!(position(&reader), before);
        let retry = encoded(|writer| header(writer, 4, None));
        let mut retry_reader = BodyReader::try_new(&retry, retry.len()).expect("Retry body");
        assert_eq!(
            present(
                owner
                    .prepare(&mut retry_reader)
                    .expect("Retry header")
                    .expect("Record")
                    .decode(Some(&prior))
                    .expect("Retry")
            ),
            prior
        );
    }
}

#[test]
fn duplicate_descending_and_overflow_indices_preserve_accepted_predecessor() {
    for next in [62, 63, 64] {
        let bytes = encoded(|writer| {
            header(writer, 63, None);
            if next == 64 {
                writer
                    .write_bits(3, 2)
                    .expect("More plus increment overflow");
            } else {
                header(writer, next, None);
            }
        });
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let mut owner = ClientStateDecoder::new(PROFILES[1]);
        assert_eq!(
            present(
                owner
                    .prepare(&mut reader)
                    .expect("First")
                    .expect("Record")
                    .decode(None)
                    .expect("First state")
            )
            .number(),
            63
        );
        let before = position(&reader);
        assert!(owner.prepare(&mut reader).is_err());
        assert_eq!(position(&reader), before);
        // End framing remains usable, proving failure did not end the stream.
        let mut end = BodyReader::try_new(&[0], 1).expect("End body");
        assert!(owner.prepare(&mut end).expect("End").is_none());
    }
}

#[test]
fn cancellation_includes_header_and_empty_input_does_not_end_owner() {
    let bytes = [0x15, 0];
    let mut owner = ClientStateDecoder::new(PROFILES[1]);
    let mut empty = BodyReader::try_new(&[], 0).expect("Empty");
    assert!(owner.prepare(&mut empty).is_err());
    let mut reader = BodyReader::try_new(&bytes, 2).expect("Body");
    let before = position(&reader);
    for _ in 0..3 {
        let prepared = owner.prepare(&mut reader).expect("Header").expect("Record");
        assert_eq!(prepared.number(), 5);
        drop(prepared);
        assert_eq!(position(&reader), before);
    }
    assert_eq!(
        present(
            owner
                .prepare(&mut reader)
                .expect("Header")
                .expect("Record")
                .decode(None)
                .expect("Commit")
        )
        .number(),
        5
    );
}
