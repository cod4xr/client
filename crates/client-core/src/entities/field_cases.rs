// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Independent finite native field vectors."
)]
use super::{BaselineWord, fields};
use crate::message_body::BodyReader;

fn vectors() -> Vec<(i16, &'static [u8], u32)> {
    vec![
        (0, &[0], 0),
        (0, &[2], 0x8000_0000),
        (0, &[3, 0x34, 0x12, 0xc0, 0x7f], 0x7fc0_1234),
        (0, &[5, 0], 1.0_f32.to_bits()),
        (0, &[0x7d, 0xff], (-1.0_f32).to_bits()),
        (0, &[0x7d, 0x7f], 4095.0_f32.to_bits()),
        (0, &[1, 0x80], (-4096.0_f32).to_bits()),
        (1, &[3], 1),
        (2, &[7], 3),
        (3, &[11], 5),
        (4, &[21], 10),
        (7, &[0xcb], 0x65),
        (8, &[1, 0xa5], 0xa5),
        (10, &[7, 0xaa], 0x2ab),
        (24, &[1, 0x56, 0x34, 0x12], 0x0012_3456),
        (32, &[1, 0xef, 0xcd, 0xab, 0x89], 0x89ab_cdef),
        (-8, &[1, 0xfd], 0xffff_fffd),
        (-8, &[0], 0),
        (-100, &[0], 0),
        (-100, &[1, 0, 0x40], 90.0_f32.to_bits()),
        (-100, &[1, 0, 0x80], (-180.0_f32).to_bits()),
        (-100, &[1, 0, 0xc0], (-90.0_f32).to_bits()),
        (-98, &[0], 1),
        (-98, &[0x3e], 0x8000_0000),
        (-98, &[1, 0x56, 0x34, 0x12], 0x0012_3456),
        (-97, &[0x4a, 0], 0xffff_ffdb),
        (-97, &[1, 0x34, 0x12, 0xcd, 0xab], 0xabcd_1234),
        (-96, &[1], 1022),
        (-96, &[2], 0),
        (-96, &[0x0c, 0xaa], 0x2ab),
        (-94, &[0xa5], 0xa5),
        (-93, &[0xa5], 0xa5),
        (-92, &[0x34, 0x12, 0xc0, 0x7f], 0x7fc0_1234),
        (-91, &[0x34, 0x12, 0xc0, 0x7f], 0x7fc0_1234),
        (-90, &[0x34, 0x12, 0xc0, 0x7f], 0x7fc0_1234),
    ]
}

#[test]
fn independent_zero_reference_field_vectors_preserve_native_bit_and_byte_forms() {
    for (bits, input, expected) in vectors() {
        let mut reader = BodyReader::try_new(input, input.len()).expect("Field");
        assert_eq!(
            fields::read(&mut reader, fields::f(BaselineWord::Union0, bits, true)),
            Ok(expected)
        );
        assert_eq!(reader.remaining_bytes(), 0);
    }
    for bits in [1, 2, 3, 4, 7, 8, 10, 24, 32] {
        let mut reader = BodyReader::try_new(&[0], 1).expect("Zero integer");
        assert_eq!(
            fields::read(&mut reader, fields::f(BaselineWord::Union0, bits, true)),
            Ok(0)
        );
    }
}

#[test]
fn every_short_field_vector_rejects_and_outer_change_flags_gate_the_whole_value() {
    for (bits, input, _) in vectors() {
        for end in 0..input.len() {
            let mut reader = BodyReader::try_new(&input[..end], input.len()).expect("Short field");
            assert!(
                fields::read(&mut reader, fields::f(BaselineWord::Union0, bits, true)).is_err()
            );
        }
        let mut reader = BodyReader::try_new(&[0], 1).expect("Unchanged flag");
        assert_eq!(
            fields::read(&mut reader, fields::f(BaselineWord::Union0, bits, false)),
            Ok(0)
        );
        assert_eq!(reader.remaining_cached_bits(), 7);
    }
    let mut reader = BodyReader::try_new(&[1, 0xa5], 2).expect("Changed flag then raw event byte");
    assert_eq!(
        fields::read(&mut reader, fields::f(BaselineWord::Event0, -94, false)),
        Ok(0xa5)
    );
}
