// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn every_truncated_legacy_baseline_preserves_both_reader_and_index_before_complete_retry() {
    let source = fixture(
        17,
        [
            Coordinate::Small(127),
            Coordinate::Full(0xffff),
            Coordinate::Full(0x8000),
        ],
        false,
        false,
    );
    for end in 0..source.len() {
        let mut stream = decoder([1000.0, -1000.0, 0.0]);
        let mut reader = BodyReader::try_new(&source[..end], source.len()).expect("Truncated");
        let before = position(&reader);
        assert!(stream.parse(&mut reader).is_err());
        assert_eq!(position(&reader), before);
        let mut retry = BodyReader::try_new(&source, source.len()).expect("Complete retry");
        let baseline = stream.parse(&mut retry).expect("Same index after failure");
        assert_eq!(baseline.word(W::PositionY), (-1999.0_f32).to_bits());
        assert_eq!(baseline.word(W::PositionZ), (-32_768.0_f32).to_bits());
    }
}

#[test]
fn late_legacy_coordinate_failure_keeps_an_existing_cached_prefix_and_prior_stream_number() {
    let first = fixture(16, [Coordinate::Small(64); 3], false, false);
    let complete = fixture(17, [Coordinate::Full(0xffff); 3], true, false);
    let mut stream = decoder([1000.0, -1000.0, 0.0]);
    let mut first_reader = BodyReader::try_new(&first, first.len()).expect("First");
    stream.parse(&mut first_reader).expect("Existing stream");
    let mut reader = BodyReader::try_new(&complete[..complete.len() - 1], complete.len())
        .expect("Late truncated coordinate");
    assert_eq!(reader.read_bits(3), Ok(5));
    let mut expected = reader.clone();
    let before = position(&reader);
    assert!(stream.parse(&mut reader).is_err());
    assert_eq!(position(&reader), before);
    assert_eq!(reader.read_bits(5), expected.read_bits(5));
    let mut retry = BodyReader::try_new(&complete, complete.len()).expect("Complete retry");
    assert_eq!(retry.read_bits(3), Ok(5));
    assert_eq!(
        stream.parse(&mut retry).expect("Same next number").number(),
        17
    );
}

#[test]
fn full_context_range_uses_defined_wrapping_without_exposing_biases_in_debug() {
    let source = fixture(17, [Coordinate::Full(0xffff); 3], false, false);
    for center in [[-2_147_483_648.0; 3], [2_147_483_520.0; 3]] {
        let mut stream = decoder(center);
        let mut reader = BodyReader::try_new(&source, source.len()).expect("Body");
        let baseline = stream
            .parse(&mut reader)
            .expect("Defined native integer arithmetic");
        let expected = if center[0].is_sign_negative() {
            1.0_f32
        } else {
            -257.0_f32
        };
        assert_eq!(baseline.word(W::PositionX), expected.to_bits());
        assert!(!format!("{stream:?}").contains("214748"));
    }
}
