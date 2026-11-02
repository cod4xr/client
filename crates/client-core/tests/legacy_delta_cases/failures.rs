// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn late_invalid_coordinate_reference_preserves_both_owners_and_all_prior_state_words() {
    let prior = raw_reference(17, [1.75_f32.to_bits(), 0x7fc0_1234, 2.5_f32.to_bits()]);
    let bytes = fixture(
        17,
        [
            Coordinate::Small(65),
            Coordinate::Full(0),
            Coordinate::Small(64),
        ],
        false,
        false,
    );
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    let before = position(&reader);
    let mut stream = decoder([1000.0; 3]);
    assert_eq!(
        stream
            .prepare(&mut reader)
            .expect("Index")
            .decode(Some(&prior), 0),
        Err(EntityDeltaError::ReferenceCoordinate)
    );
    assert_eq!(position(&reader), before);
    assert_eq!(prior.word(W::PositionX), 1.75_f32.to_bits());
    assert_eq!(prior.word(W::PositionY), 0x7fc0_1234);
    let valid = reference(17);
    assert_eq!(
        present(
            stream
                .prepare(&mut reader)
                .expect("Same number")
                .decode(Some(&valid), 0)
                .expect("Valid reference retry")
        )
        .word(W::PositionX),
        2.0_f32.to_bits()
    );
}

#[test]
fn all_truncations_and_cancelled_preparation_preserve_a_partial_reader_and_index() {
    let prior = reference(17);
    let bytes = fixture(
        17,
        [
            Coordinate::Small(65),
            Coordinate::Full(0xffff),
            Coordinate::Full(0x8000),
        ],
        true,
        false,
    );
    for end in 1..bytes.len() {
        let mut stream = decoder([1000.0, -1000.0, 0.0]);
        let mut reader = BodyReader::try_new(&bytes[..end], bytes.len()).expect("Truncated");
        assert_eq!(reader.read_bits(3), Ok(5));
        let before = position(&reader);
        let mut expected = reader.clone();
        let result = stream
            .prepare(&mut reader)
            .and_then(|pending| pending.decode(Some(&prior), 0));
        assert!(result.is_err());
        assert_eq!(position(&reader), before);
        assert_eq!(reader.read_bits(5), expected.read_bits(5));
        let mut retry = BodyReader::try_new(&bytes, bytes.len()).expect("Complete retry");
        assert_eq!(retry.read_bits(3), Ok(5));
        let pending = stream.prepare(&mut retry).expect("Same index");
        assert_eq!(pending.number(), 17);
        drop(pending);
        let _ = stream
            .prepare(&mut retry)
            .expect("Cancellation keeps same index")
            .decode(Some(&prior), 0)
            .expect("Complete record");
    }
}

#[test]
fn full_forms_reject_out_of_range_priors_but_finite_short_forms_keep_native_float_addition() {
    let full = fixture(17, [Coordinate::Full(0); 3], false, false);
    let short = fixture(17, [Coordinate::Small(65); 3], false, false);
    for value in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        2_147_483_648.0,
        -2_147_483_904.0,
        1.0e30,
    ] {
        let prior = raw_reference(17, [value.to_bits(); 3]);
        let mut stream = decoder([0.0; 3]);
        let mut reader = BodyReader::try_new(&full, full.len()).expect("Full");
        let before = position(&reader);
        assert_eq!(
            stream
                .prepare(&mut reader)
                .expect("Index")
                .decode(Some(&prior), 0),
            Err(EntityDeltaError::ReferenceCoordinate)
        );
        assert_eq!(position(&reader), before);
        let mut reader = BodyReader::try_new(&short, short.len()).expect("Short");
        let result = stream
            .prepare(&mut reader)
            .expect("Same index")
            .decode(Some(&prior), 0);
        if value.is_finite() {
            assert_eq!(
                present(result.expect("No integer conversion")).word(W::PositionX),
                (value + 1.0).to_bits()
            );
        } else {
            assert_eq!(result, Err(EntityDeltaError::ReferenceCoordinate));
        }
    }
}
