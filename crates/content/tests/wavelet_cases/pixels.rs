// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn all_five_encodings_reconstruct_and_preserve_dense_channel_order() {
    for (code, seed, expected, format) in [
        (
            6,
            &[7, 11, 13, 17][..],
            &[7, 11, 13, 17][..],
            Iwi6Format::Bgra8,
        ),
        (
            7,
            &[7, 11, 13][..],
            &[7, 11, 13, 255][..],
            Iwi6Format::Bgra8,
        ),
        (8, &[7, 17][..], &[7, 17][..], Iwi6Format::LuminanceAlpha8),
        (9, &[7][..], &[7][..], Iwi6Format::Luminance8),
        (10, &[17][..], &[17][..], Iwi6Format::Alpha8),
    ] {
        let mut stream = Stream::raw(seed);
        stream.put(0, 1); // no delta on parent
        stream.triplet(seed.len());
        let input = fixture(code, 0, 2, 2, &stream.bytes);
        let mut scratch = [0; 128];
        let mut output = [0xa5; 128];
        let image = Iwi6WaveletImage::decode(&input, BUDGET, &mut scratch, &mut output)
            .expect("complete entropy vector");
        assert_eq!(image.format(), format);
        assert_eq!(image.mip(1).expect("seed").bytes(), expected);
        assert_eq!(image.mip(0).expect("base").bytes(), expected.repeat(4));
        assert!(image.mip(2).is_none());
        assert_eq!(
            image.consumed(),
            (
                stream.bits / 8,
                u8::try_from(stream.bits % 8).expect("bit offset")
            )
        );
        assert_eq!(image.encoded_bytes(), stream.bytes.len());
    }
}

#[test]
fn coefficients_parity_negative_shifts_and_modulo_wrap_match_hand_vector() {
    let mut stream = Stream::raw(&[250]);
    stream.put(0, 1);
    stream.put(1, 1); // top-left parity
    for value in [21, -17, 9] {
        stream.alpha(value);
    }
    let input = fixture(10, 0, 2, 2, &stream.bytes);
    let mut scratch = [0; 6];
    let mut output = [0; 5];
    let image = Iwi6WaveletImage::decode(&input, BUDGET, &mut scratch, &mut output)
        .expect("signed coefficients");
    // (500+21-17+9)>>1 + parity, (500+21+17-9)>>1,
    // (500-17-9-21)>>1, (500-21+17+9)>>1, reduced modulo 256.
    assert_eq!(image.mip(0).expect("base").bytes(), [1, 8, 226, 252]);
    let mut negative = Stream::raw(&[0]);
    negative.put(0, 1);
    negative.put(0, 1);
    for value in [-3, 0, 0] {
        negative.alpha(value);
    }
    let input = fixture(9, 0, 2, 2, &negative.bytes);
    let image = Iwi6WaveletImage::decode(&input, BUDGET, &mut scratch, &mut output)
        .expect("arithmetic signed shifts");
    assert_eq!(image.mip(0).expect("base").bytes(), [254, 254, 1, 1]);
}

#[test]
fn source_delta_is_private_and_original_smaller_mip_survives() {
    let mut stream = Stream::raw(&[254]);
    stream.put(1, 1);
    stream.alpha(3); // parent wraps to 1 privately
    stream.triplet(1);
    let input = fixture(9, 0, 2, 2, &stream.bytes);
    let mut scratch = [0; 6];
    let mut output = [0; 5];
    let image = Iwi6WaveletImage::decode(&input, BUDGET, &mut scratch, &mut output).expect("delta");
    assert_eq!(image.mip(1).expect("original seed").bytes(), [254]);
    assert_eq!(image.mip(0).expect("base").bytes(), [1; 4]);
}

#[test]
fn cube_faces_and_rectangular_raw_prefix_levels_follow_physical_order() {
    let mut stream = Stream::raw(&[1, 2, 3, 4, 5, 6]);
    for _ in 0..6 {
        stream.put(0, 1);
        stream.triplet(1);
    }
    let cube = fixture(9, 4, 2, 2, &stream.bytes);
    let mut scratch = [0; 128];
    let mut output = [0; 128];
    let image =
        Iwi6WaveletImage::decode(&cube, BUDGET, &mut scratch, &mut output).expect("six faces");
    for face in 0..6 {
        assert_eq!(
            image.mip(0).expect("base").face(face),
            Some([face + 1; 4].as_slice())
        );
    }
    let mut stream = Stream::raw(&[99, 10, 20]); // 1x1, then raw 1x2 mip
    stream.put(0, 1);
    stream.triplet(1);
    stream.triplet(1);
    let rectangle = fixture(9, 0, 2, 4, &stream.bytes);
    let image =
        Iwi6WaveletImage::decode(&rectangle, BUDGET, &mut scratch, &mut output).expect("rectangle");
    assert_eq!(image.mip(2).expect("seed").bytes(), [99]);
    assert_eq!(image.mip(1).expect("raw line").bytes(), [10, 20]);
    assert_eq!(
        image.mip(0).expect("base").bytes(),
        [10, 10, 10, 10, 20, 20, 20, 20]
    );
}

#[test]
fn returned_pixels_outlive_input_work_and_image_plan_without_private_debug() {
    let mut output = [0; 64];
    let mip = {
        let input = fixture(6, 2, 1, 1, b"Hide");
        let mut scratch = [0; 4];
        let image =
            Iwi6WaveletImage::decode(&input, BUDGET, &mut scratch, &mut output).expect("raw pixel");
        let debug = format!("{image:?}");
        assert!(!debug.contains("Hide"));
        assert!(!debug.contains("[72, 105"));
        image.mip(0).expect("owned output lifetime")
    };
    assert_eq!(mip.bytes(), b"Hide");
    assert_eq!(output[4..], [0; 60]);
}
