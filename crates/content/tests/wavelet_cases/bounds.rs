// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn every_stream_truncation_and_short_capacity_preserves_entire_output() {
    let mut stream = Stream::raw(&[8, 16, 32, 64]);
    stream.put(1, 1);
    for delta in [2, -3, 4, -5] {
        stream.alpha(delta);
    }
    stream.triplet(4);
    let input = fixture(6, 0, 2, 2, &stream.bytes);
    let mut scratch = [0; 24];
    let mut output = [0xa5; 28];
    for end in 0..input.len() {
        assert!(
            Iwi6WaveletImage::decode(&input[..end], BUDGET, &mut scratch, &mut output).is_err()
        );
        assert_eq!(output, [0xa5; 28]);
    }
    for length in 0..24 {
        assert_eq!(
            Iwi6WaveletImage::decode(&input, BUDGET, &mut scratch[..length], &mut output).err(),
            Some(WaveletError::Capacity)
        );
        assert_eq!(output, [0xa5; 28]);
    }
    for length in 0..20 {
        assert_eq!(
            Iwi6WaveletImage::decode(&input, BUDGET, &mut scratch, &mut output[..length]).err(),
            Some(WaveletError::Capacity)
        );
        assert_eq!(output, [0xa5; 28]);
    }
    let image = Iwi6WaveletImage::decode(&input, BUDGET, &mut scratch, &mut output)
        .expect("exact capacities");
    assert_eq!(image.decoded_bytes(), 20);
    assert_eq!(output[20..], [0xa5; 8]);
}

#[test]
fn explicit_budgets_shape_and_format_reject_before_publication() {
    let input = fixture(9, 0, 2, 2, &[10, 0x3c]); // zero alpha coefficients
    let need = Iwi6WaveletImage::requirements(&input, BUDGET).expect("metadata");
    assert_eq!((need.decoded_bytes, need.scratch_bytes), (5, 6));
    for (budget, error) in [
        (
            Iwi6WaveletBudget {
                max_decoded_bytes: 4,
                ..BUDGET
            },
            WaveletError::DecodedBudget,
        ),
        (
            Iwi6WaveletBudget {
                max_scratch_bytes: 5,
                ..BUDGET
            },
            WaveletError::ScratchBudget,
        ),
    ] {
        let mut scratch = [0; 6];
        let mut output = [0x55; 5];
        assert_eq!(
            Iwi6WaveletImage::decode(&input, budget, &mut scratch, &mut output).err(),
            Some(error)
        );
        assert_eq!(output, [0x55; 5]);
    }
    for (code, flags, width, height, error) in [
        (1, 0, 2, 2, WaveletError::Format),
        (9, 8, 2, 2, WaveletError::Geometry),
        (9, 0, 3, 2, WaveletError::Geometry),
        (9, 2, 2, 2, WaveletError::Geometry),
    ] {
        assert_eq!(
            Iwi6WaveletImage::requirements(&fixture(code, flags, width, height, &[]), BUDGET),
            Err(error)
        );
    }
}

#[test]
fn raw_only_line_and_trailing_entropy_are_explicitly_reported() {
    let mut input = fixture(9, 2, 1, 2, &[11, 22]);
    input.extend_from_slice(&[0xde, 0xad]);
    let mut scratch = [0; 2];
    let mut output = [0; 2];
    let image = Iwi6WaveletImage::decode(&input, BUDGET, &mut scratch, &mut output)
        .expect("raw line with inert tail");
    assert_eq!(image.mip(0).expect("line").bytes(), [11, 22]);
    assert_eq!(image.consumed(), (2, 0));
    assert_eq!(image.encoded_bytes(), 4);
}

#[test]
fn deterministic_entropy_mutations_are_bounded_and_publish_only_complete_images() {
    let input = fixture(6, 0, 2, 2, &[17, 31, 63, 127, 0x24, 0x49, 0xff, 0xff, 0x7f]);
    for index in 28..input.len() {
        for value in 0..=255 {
            let mut bytes = input.clone();
            bytes[index] = value;
            let mut scratch = [0; 24];
            let mut output = [0xa5; 28];
            match Iwi6WaveletImage::decode(&bytes, BUDGET, &mut scratch, &mut output) {
                Ok(image) => assert_eq!(image.decoded_bytes(), 20),
                Err(_) => assert_eq!(output, [0xa5; 28]),
            }
            assert_eq!(output[20..], [0xa5; 8]);
        }
    }
}
