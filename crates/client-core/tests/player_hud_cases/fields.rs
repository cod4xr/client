// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{FIELDS, MODERN, W, integer, legacy, one_field, raw_float, seed};

const SCALES: [u32; 32] = [
    0x3fb3_3333,
    0x3fc0_0000,
    0x3fcc_cccd,
    0x3fd9_9999,
    0x3fe6_6666,
    0x3ff3_3333,
    0x4000_0000,
    0x4006_6666,
    0x400c_cccd,
    0x4013_3333,
    0x4019_999a,
    0x4020_0000,
    0x4026_6666,
    0x402c_cccd,
    0x4033_3333,
    0x4039_999a,
    0x4040_0000,
    0x4046_6666,
    0x404c_cccd,
    0x4053_3333,
    0x4059_999a,
    0x4060_0000,
    0x4066_6666,
    0x406c_cccd,
    0x4073_3333,
    0x4079_999a,
    0x4080_0000,
    0x4083_3333,
    0x4086_6666,
    0x4089_999a,
    0x408c_cccd,
    0x4090_0000,
];

#[test]
fn all_forty_native_field_rules_map_to_distinct_owned_semantic_words() {
    for &(word, bits) in &FIELDS {
        let expected = match bits {
            -85 => 0xff00_0000,
            -97 => 7_u32.wrapping_sub(37),
            -92..=-90 => 0x7fc0_4321,
            0 => 0x7fc0_1234,
            -86 => 0x3fd9_9999,
            -99 => 2.0_f32.to_bits(),
            _ => 0x89ab_cdef & (u32::MAX >> (32 - u32::try_from(bits).expect("Width"))),
        };
        let result = one_field(MODERN, 7, None, word, |writer| match bits {
            -85 => writer.write_bits(3, 2).expect("Changed color shortcut"),
            -97 => {
                writer.write_bits(1, 2).expect("Changed relative time");
                writer.write_bits(37, 8).expect("Age");
            }
            -92..=-90 => {
                writer.write_bits(1, 1).expect("Changed absolute origin");
                writer
                    .write_i32(expected.cast_signed())
                    .expect("Raw origin");
            }
            0 => raw_float(writer, 0, expected),
            -86 => {
                writer.write_bits(1, 1).expect("Changed scale");
                writer.write_bits(3, 5).expect("Scale index");
            }
            -99 => {
                writer.write_bits(3, 3).expect("Changed compact coordinate");
                writer.write_bits(2, 4).expect("Low biased XOR");
                writer.write_u8(0).expect("High biased XOR");
            }
            _ => integer(writer, u8::try_from(bits).expect("Width"), 0, expected),
        });
        assert_eq!(result.archival()[0].word(word), expected, "{word:?}");
        for &(other, _) in &FIELDS {
            if other != word {
                assert_eq!(result.archival()[0].word(other), 0, "{other:?}");
            }
        }
    }
}

#[test]
fn every_scale_index_matches_independent_native_coefficient_goldens() {
    for context in [MODERN, legacy([0.0; 3])] {
        for (index, &expected) in SCALES.iter().enumerate() {
            let result = one_field(context, 0, None, W::FontScale, |writer| {
                writer.write_bits(1, 1).expect("Changed scale");
                writer
                    .write_bits(u32::try_from(index).expect("Index"), 5)
                    .expect("Native scale bits");
            });
            assert_eq!(
                result.archival()[0].word(W::FontScale),
                expected,
                "index {index}"
            );
        }
    }
}

fn color_prior(context: super::EntityDecodeContext, alpha: u32) -> super::SnapshotHud {
    one_field(context, 0, None, W::Color, |writer| {
        writer.write_bits(1, 1).expect("Changed color");
        writer.write_bits(0, 2).expect("New absolute RGB");
        for byte in [0x33, 0x22, 0x11] {
            writer.write_u8(byte).expect("RGB byte");
        }
        writer.write_bits(alpha, 5).expect("Quantized alpha");
    })
}

#[test]
fn packed_color_shortcut_clears_any_nonzero_alpha_and_preserves_rgb() {
    for context in [MODERN, legacy([0.0; 3])] {
        for alpha in [0, 1, 31] {
            let prior = color_prior(context, alpha);
            let result = one_field(context, 0, Some(&prior), W::Color, |writer| {
                writer.write_bits(3, 2).expect("Changed color shortcut");
            });
            assert_eq!(
                result.archival()[0].word(W::Color),
                if alpha == 0 { 0xff11_2233 } else { 0x0011_2233 }
            );
        }
    }
}

#[test]
fn full_colors_quantize_all_alphas_keep_or_replace_rgb_absolutely() {
    let prior = color_prior(MODERN, 31);
    for alpha in 0..32 {
        for keep in [true, false] {
            let result = one_field(MODERN, 0, Some(&prior), W::Color, |writer| {
                writer.write_bits(1, 1).expect("Changed color");
                writer.write_bits(0, 1).expect("Full color form");
                writer
                    .write_bits(u32::from(keep), 1)
                    .expect("Keep RGB flag");
                if !keep {
                    for byte in [0xaa, 0xbb, 0xcc] {
                        writer.write_u8(byte).expect("New RGB");
                    }
                }
                writer
                    .write_bits(alpha, 5)
                    .expect("Alpha after byte operations");
            });
            assert_eq!(
                result.archival()[0].word(W::Color),
                (alpha << 27) | if keep { 0x0011_2233 } else { 0x00cc_bbaa }
            );
        }
    }
}

#[test]
fn compact_coordinates_keep_high_prior_bias_bits_and_native_signed_wrap() {
    for (prior_bits, low, high, expected) in [
        (1.75_f32.to_bits(), 15, 255, (-2.0_f32).to_bits()),
        ((-1.75_f32).to_bits(), 13, 255, 2.0_f32.to_bits()),
        (4096.0_f32.to_bits(), 15, 255, 4095.0_f32.to_bits()),
        ((-4096.0_f32).to_bits(), 15, 255, (-4097.0_f32).to_bits()),
        ((-2_147_483_648.0_f32).to_bits(), 15, 255, 0x4f00_0000),
        (2_147_483_520.0_f32.to_bits(), 15, 255, 0xceff_ffff),
    ] {
        let prior = seed(MODERN, &[1], prior_bits);
        let result = one_field(MODERN, 0, Some(&prior), W::FromX, |writer| {
            writer.write_bits(3, 3).expect("Changed compact coordinate");
            writer.write_bits(low, 4).expect("Low window");
            writer.write_u8(high).expect("High window");
        });
        assert_eq!(result.archival()[0].word(W::FromX), expected);
        assert_eq!(result.archival()[0].word(W::Value), prior_bits);
    }
}

#[test]
fn compact_zero_and_raw_forms_preserve_their_distinct_nan_and_sign_rules() {
    let prior = seed(MODERN, &[1], 0x7fc0_1234);
    let zero = one_field(MODERN, 0, Some(&prior), W::FromX, |writer| {
        writer.write_bits(1, 2).expect("Changed positive zero");
    });
    assert_eq!(zero.archival()[0].word(W::FromX), 0);
    let raw = one_field(MODERN, 0, Some(&prior), W::FromX, |writer| {
        raw_float(writer, 0x7fc0_1234, 0x8000_0000);
    });
    assert_eq!(raw.archival()[0].word(W::FromX), 0x8000_0000);
    let raw = one_field(MODERN, 0, Some(&raw), W::FromX, |writer| {
        raw_float(writer, 0x8000_0000, 0x7fc0_4321);
    });
    assert_eq!(raw.archival()[0].word(W::FromX), 0x7fc0_4321);
    let signed_zero = one_field(MODERN, 0, Some(&prior), W::Value, |writer| {
        writer
            .write_bits(5, 3)
            .expect("Changed ordinary negative zero");
    });
    assert_eq!(signed_zero.archival()[0].word(W::Value), 0x8000_0000);
}

#[test]
fn modern_origin_words_are_absolute_rather_than_xor_deltas() {
    for word in [W::X, W::Y, W::Z] {
        let prior = one_field(MODERN, 0, None, word, |writer| {
            writer.write_bits(1, 1).expect("Changed origin");
            writer
                .write_i32(1.5_f32.to_bits().cast_signed())
                .expect("Raw prior");
        });
        let result = one_field(MODERN, 0, Some(&prior), word, |writer| {
            writer.write_bits(1, 1).expect("Changed origin");
            writer.write_i32(0x7fc0_4321).expect("Raw target");
        });
        assert_eq!(result.archival()[0].word(word), 0x7fc0_4321);
    }
}
