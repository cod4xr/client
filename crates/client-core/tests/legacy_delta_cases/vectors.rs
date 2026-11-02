// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn full_forms_preserve_native_signed_wrap_and_binary32_rounding_at_integer_boundaries() {
    for (value, expected) in [
        (2_147_483_520.0_f32, (-2_147_483_520.0_f32).to_bits()),
        (-2_147_483_648.0, 2_147_483_648.0_f32.to_bits()),
        (16_777_216.0, 16_777_215.0_f32.to_bits()),
        (-16_777_216.0, (-16_777_216.0_f32).to_bits()),
    ] {
        let prior = raw_reference(17, [value.to_bits(); 3]);
        let source = fixture(17, [Coordinate::Full(0xffff); 3], false, false);
        let mut reader = BodyReader::try_new(&source, source.len()).expect("Body");
        let state = present(
            decoder([0.0; 3])
                .prepare(&mut reader)
                .expect("Index")
                .decode(Some(&prior), 0)
                .expect("Defined native coordinate arithmetic"),
        );
        for word in [W::PositionX, W::PositionY, W::PositionZ] {
            assert_eq!(state.word(word), expected);
        }
    }
}

#[test]
fn short_forms_add_to_each_prior_float_and_full_forms_truncate_before_native_bias_xor() {
    let prior = raw_reference(
        17,
        [1.75_f32.to_bits(), (-1.75_f32).to_bits(), 2.5_f32.to_bits()],
    );
    for (coordinates, expected) in [
        (
            [
                Coordinate::Small(65),
                Coordinate::Small(63),
                Coordinate::Small(64),
            ],
            [2.75_f32, -2.75, 2.5],
        ),
        (
            [
                Coordinate::Full(0),
                Coordinate::Full(0),
                Coordinate::Full(0),
            ],
            [1.0, -1.0, 2.0],
        ),
        (
            [
                Coordinate::Full(1),
                Coordinate::Full(1),
                Coordinate::Full(1),
            ],
            [0.0, 0.0, 3.0],
        ),
    ] {
        let bytes = fixture(17, coordinates, false, true);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let state = present(
            decoder([1000.0, -1000.0, 0.0])
                .prepare(&mut reader)
                .expect("Index")
                .decode(Some(&prior), 0)
                .expect("Complete legacy update"),
        );
        for (word, expected) in [W::PositionX, W::PositionY, W::PositionZ]
            .into_iter()
            .zip(expected)
        {
            assert_eq!(state.word(word), expected.to_bits());
        }
        assert_eq!(reader.read_u8(), Ok(7));
        assert_eq!(reader.read_bits(5), Ok(21));
    }
}

#[test]
fn every_short_delta_value_updates_nonzero_prior_words_without_consulting_map_center() {
    let prior = reference(17);
    for encoded in 0..128_u8 {
        let bytes = fixture(17, [Coordinate::Small(encoded); 3], false, false);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let state = present(
            decoder([1_000_000.0, -1_000_000.0, 0.0])
                .prepare(&mut reader)
                .expect("Index")
                .decode(Some(&prior), u32::MAX)
                .expect("Legacy origin delta"),
        );
        for (word, base) in [(W::PositionX, 1_i16), (W::PositionY, -1), (W::PositionZ, 2)] {
            assert_eq!(
                state.word(word),
                f32::from(base + i16::from(encoded) - 64).to_bits()
            );
        }
    }
}

#[test]
fn no_delta_and_removal_preserve_or_discard_owned_raw_state_without_coordinate_math() {
    let prior = raw_reference(0, [0x7fc0_1234, f32::INFINITY.to_bits(), 1.0_f32.to_bits()]);
    let mut stream = decoder([1000.0; 3]);
    let mut reader = BodyReader::try_new(&[0xa1, 7], 2).expect("No-delta record");
    let state = present(
        stream
            .prepare(&mut reader)
            .expect("Index")
            .decode(Some(&prior), 0)
            .expect("Exact retained state"),
    );
    assert_eq!(state, prior);
    assert_eq!(reader.read_u8(), Ok(7));
    let mut reader = BodyReader::try_new(&[0x82, 0x10], 2).expect("Removal 32");
    assert_eq!(
        stream
            .prepare(&mut reader)
            .expect("Later index")
            .decode(None, 0),
        Ok(EntityDelta::Removed { number: 32 })
    );
}

#[test]
fn source_and_prior_release_do_not_invalidate_the_owned_legacy_result() {
    let mut bytes = fixture(17, [Coordinate::Small(65); 3], false, false);
    let state = {
        let prior = reference(17);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        present(
            decoder([0.0; 3])
                .prepare(&mut reader)
                .expect("Index")
                .decode(Some(&prior), 0)
                .expect("Owned state"),
        )
    };
    bytes.fill(0);
    assert_eq!(state.word(W::PositionX), 2.0_f32.to_bits());
    assert_eq!(state.word(W::PositionY), 0.0_f32.to_bits());
    assert_eq!(state.word(W::PositionZ), 3.0_f32.to_bits());
    assert!(!format!("{state:?}").contains("1073741824"));
}
