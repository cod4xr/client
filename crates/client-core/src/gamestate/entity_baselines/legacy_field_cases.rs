// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Independent finite legacy coordinate bytes."
)]
use super::{LegacyMapCenter, legacy_coordinates};
use crate::message_body::BodyReader;

#[test]
fn physical_legacy_coordinate_vectors_use_all_three_axes_and_signed_native_biases() {
    let center = LegacyMapCenter::new([1000.0, -1000.0, 0.0]).expect("Context");
    for (input, expected) in [
        (b"\0".as_slice(), [-64.0_f32; 3]),
        (&[0x7e], [-1.0; 3]),
        (&[0x80], [0.0; 3]),
        (&[0xfe], [63.0; 3]),
        (&[1, 0, 0], [0.0; 3]),
        (&[0xff, 0xff, 1], [1999.0, -1999.0, -1.0]),
        (&[1, 0, 1], [32_768.0, -32_768.0, -32_768.0]),
    ] {
        for (bits, expected) in [-92, -91, -90].into_iter().zip(expected) {
            let mut reader = BodyReader::try_new(input, input.len()).expect("Physical field");
            assert_eq!(
                legacy_coordinates::read_zero(&mut reader, bits, center).expect("Legacy field"),
                expected.to_bits()
            );
            assert_eq!(reader.remaining_bytes(), 0);
        }
    }
}

#[test]
fn every_truncated_physical_coordinate_vector_rejects_without_guessing_a_wider_profile() {
    let center = LegacyMapCenter::new([0.0; 3]).expect("Context");
    for input in [b"\xfe".as_slice(), &[0xff, 0xff, 1], &[1, 0, 1]] {
        for end in 0..input.len() {
            let mut reader = BodyReader::try_new(&input[..end], input.len()).expect("Truncated");
            assert!(legacy_coordinates::read_zero(&mut reader, -92, center).is_err());
        }
    }
}
