// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn correlated_color_coefficients_use_blue_reference_and_independent_parity() {
    let mut stream = Stream::raw(&[10, 20, 30, 40]);
    stream.put(0, 1); // unchanged source mip
    stream.put(1, 1); // blue parity
    for code in [0x005, 0x015, 0x004] {
        stream.put(code, 5);
    } // 2,-2,4
    stream.put(0, 1); // green parity
    stream.put(0x002, 3);
    stream.put(0x006, 3);
    stream.put(0x001, 4); // 1,-1,2
    stream.put(1, 1); // red parity
    stream.put(0x006, 3);
    stream.put(0x002, 3);
    stream.put(0x009, 4); // -1,1,-2
    stream.put(0, 1); // alpha parity
    for _ in 0..3 {
        stream.alpha(0);
    }
    let input = fixture(6, 0, 2, 2, &stream.bytes);
    let mut scratch = [0; 24];
    let mut output = [0; 20];
    let image = Iwi6WaveletImage::decode(&input, BUDGET, &mut scratch, &mut output)
        .expect("correlated color vector");
    assert_eq!(
        image.mip(0).expect("base").bytes(),
        [
            13, 23, 32, 40, 10, 20, 30, 40, 6, 14, 28, 40, 12, 23, 31, 40,
        ]
    );
    assert_eq!(image.mip(1).expect("seed").bytes(), [10, 20, 30, 40]);
}

#[test]
fn multiple_levels_and_parent_deltas_keep_each_published_mip_unchanged() {
    let mut stream = Stream::raw(&[10]);
    stream.put(0, 1);
    stream.triplet(1); // 2x2 stays ten
    stream.put(1, 1);
    for delta in [1, 2, 3, 4] {
        stream.alpha(delta);
    }
    for _ in 0..4 {
        stream.triplet(1);
    }
    let input = fixture(9, 0, 4, 4, &stream.bytes);
    let mut scratch = [0; 25];
    let mut output = [0; 21];
    let image = Iwi6WaveletImage::decode(&input, BUDGET, &mut scratch, &mut output)
        .expect("continuous multi-level entropy");
    assert_eq!(image.mip(2).expect("seed").bytes(), [10]);
    assert_eq!(image.mip(1).expect("middle").bytes(), [10; 4]);
    assert_eq!(
        image.mip(0).expect("base").bytes(),
        [
            11, 11, 12, 12, 11, 11, 12, 12, 13, 13, 14, 14, 13, 13, 14, 14,
        ]
    );
}
