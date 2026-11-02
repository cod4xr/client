// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Independent finite nonzero field vectors."
)]
use super::{BaselineWord, field_error::FieldReadError, fields};
use crate::message_body::BodyReader;

type Vector = (i16, u32, &'static [u8], u32);
fn vectors() -> Vec<Vector> {
    vec![
        (8, 0xa5, &[1, 0xff], 0x5a),
        (8, 0xffff_ffa5, &[1, 0xff], 0x5a),
        (10, 0xffff_feab, &[5, 0xff], 0x155),
        (10, 0x2ab, &[5, 0xff], 0x155),
        (32, 0x1234_5678, &[1, 0x97, 0x9b, 0x9f, 0x9b], 0x89ab_cdef),
        (-8, 0xffff_fffd, &[1, 0xff], 2),
        (-8, 2, &[1, 0xff], 0xffff_fffd),
        (-8, 0xffff_fffd, &[0], 0),
        (8, 0xa5, &[0], 0),
        (0, 1.5_f32.to_bits(), &[3, 0x34, 0x12, 0, 0x40], 0x7fc0_1234),
        (0, 0x7fc0_1234, &[3, 0x34, 0x12, 0, 0x40], 1.5_f32.to_bits()),
        (0, 1.75_f32.to_bits(), &[0x7d, 0xff], (-2.0_f32).to_bits()),
        (0, (-1.75_f32).to_bits(), &[0x75, 0xff], 2.0_f32.to_bits()),
        (
            0,
            (-4096.0_f32).to_bits(),
            &[0x7d, 0xff],
            4095.0_f32.to_bits(),
        ),
        (
            0,
            4095.0_f32.to_bits(),
            &[0x7d, 0xff],
            (-4096.0_f32).to_bits(),
        ),
        (0, 0x7fc0_1234, &[0], 0),
        (0, 1.5_f32.to_bits(), &[2], 0x8000_0000),
        (-100, 45.0_f32.to_bits(), &[1, 0, 0x40], 90.0_f32.to_bits()),
        (
            -100,
            45.0_f32.to_bits(),
            &[1, 0, 0x80],
            (-180.0_f32).to_bits(),
        ),
        (
            -100,
            45.0_f32.to_bits(),
            &[1, 0, 0xc0],
            (-90.0_f32).to_bits(),
        ),
        (-100, 45.0_f32.to_bits(), &[0], 0),
        (-98, 0xa5, &[0], 0xa4),
        (-98, 0x8000_0001, &[0x3e], 1),
        (-98, 0xff00_0000, &[1, 0x56, 0x34, 0x12], 0x0012_3456),
        (-97, 0xdead_beef, &[0x4a, 0], 0xffff_ffeb),
        (-97, 0xdead_beef, &[1, 0x34, 0x12, 0xcd, 0xab], 0xabcd_1234),
        (-96, 100, &[1], 1022),
        (-96, 100, &[2], 0),
        (-96, 100, &[0x0c, 0xaa], 0x2ab),
        (-94, 0xff, &[0xa5], 0xa5),
        (-93, 0xff, &[0xa5], 0xa5),
        (
            -92,
            1.5_f32.to_bits(),
            &[0x34, 0x12, 0xc0, 0x7f],
            0x7fc0_1234,
        ),
        (
            -91,
            1.5_f32.to_bits(),
            &[0x34, 0x12, 0xc0, 0x7f],
            0x7fc0_1234,
        ),
        (
            -90,
            1.5_f32.to_bits(),
            &[0x34, 0x12, 0xc0, 0x7f],
            0x7fc0_1234,
        ),
    ]
}

#[test]
fn independent_nonzero_field_vectors_distinguish_xor_absolute_and_message_time_forms() {
    for (bits, reference, source, expected) in vectors() {
        let mut reader = BodyReader::try_new(source, source.len()).expect("Field");
        let value = fields::read_delta(
            &mut reader,
            fields::f(BaselineWord::Union0, bits, true),
            reference,
            16,
        )
        .expect("Complete delta field");
        assert_eq!(value, expected, "encoding {bits}");
        assert_eq!(reader.remaining_bytes(), 0);
    }
}

#[test]
fn short_nonzero_fields_fail_and_unchanged_flags_preserve_the_entire_reference_word() {
    for (bits, reference, source, _) in vectors() {
        for end in 0..source.len() {
            let mut reader = BodyReader::try_new(&source[..end], source.len()).expect("Truncated");
            assert!(
                fields::read_delta(
                    &mut reader,
                    fields::f(BaselineWord::Union0, bits, true),
                    reference,
                    16
                )
                .is_err()
            );
        }
        let mut reader = BodyReader::try_new(&[0], 1).expect("Unchanged");
        let value = fields::read_delta(
            &mut reader,
            fields::f(BaselineWord::Union0, bits, false),
            reference,
            16,
        )
        .expect("Whole-word retain");
        assert_eq!(value, reference);
        assert_eq!(reader.remaining_cached_bits(), 7);
    }
}

#[test]
fn compact_float_references_require_the_writer_integer_range_while_raw_forms_keep_bits() {
    for reference in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        4096.0,
        -4097.0,
        1.0e20,
    ] {
        let mut reader = BodyReader::try_new(&[1, 0], 2).expect("Compact");
        assert!(matches!(
            fields::read_delta(
                &mut reader,
                fields::f(BaselineWord::Union0, 0, true),
                reference.to_bits(),
                0
            ),
            Err(FieldReadError::ReferenceFloat)
        ));
    }
    for (reference, expected) in [
        (4095.75_f32, 4095.0_f32.to_bits()),
        (-4096.75, (-4096.0_f32).to_bits()),
        (-0.0, 0),
    ] {
        let mut reader =
            BodyReader::try_new(&[1, 0], 2).expect("Compact unchanged integer component");
        let value = fields::read_delta(
            &mut reader,
            fields::f(BaselineWord::Union0, 0, true),
            reference.to_bits(),
            0,
        )
        .expect("Truncation before range check");
        assert_eq!(value, expected);
    }
}
