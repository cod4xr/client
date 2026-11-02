// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BaselineProtocol, EntityDecodeContext, FIELDS, MODERN, MessageWriter, W, legacy, one_field,
    parse, seed,
};

fn biases(context: EntityDecodeContext) -> [i32; 3] {
    match context {
        EntityDecodeContext::Legacy6(center) => center.biases(),
        MODERN => unreachable!("Explicit legacy fixture"),
    }
}
fn full(writer: &mut MessageWriter<'_>, bias: i32, from: i32, to: i32) {
    writer
        .write_bits(3, 2)
        .expect("Changed full legacy coordinate");
    let old = from.wrapping_sub(bias).wrapping_add(32768).cast_unsigned();
    let new = to.wrapping_sub(bias).wrapping_add(32768).cast_unsigned();
    writer
        .write_bits((old ^ new) & 65535, 16)
        .expect("Native full window");
}

#[test]
fn explicit_legacy_centers_decode_full_and_short_updates_on_each_axis() {
    let context = legacy([1000.25, -2000.75, 3000.5]);
    for (axis, (word, value)) in [(W::X, 42_i16), (W::Y, -123), (W::Z, 3333)]
        .into_iter()
        .enumerate()
    {
        let prior = one_field(context, 0, None, word, |writer| {
            full(writer, biases(context)[axis], 0, i32::from(value));
        });
        assert_eq!(prior.archival()[0].word(word), f32::from(value).to_bits());
        for delta in [-64_i16, -1, 0, 63] {
            let next = one_field(context, 0, Some(&prior), word, |writer| {
                writer.write_bits(1, 2).expect("Changed short origin");
                writer
                    .write_bits(u32::try_from(delta + 64).expect("Seven bits"), 7)
                    .expect("Relative origin");
            });
            assert_eq!(
                next.archival()[0].word(word),
                f32::from(value + delta).to_bits()
            );
        }
    }
}

#[test]
fn explicit_center_replacement_keeps_prior_words_and_uses_new_bias_for_delta_math() {
    let first = legacy([1000.25, -2000.75, 3000.5]);
    let second = legacy([2000.25, -3000.75, 4000.5]);
    let prior = one_field(first, 0, None, W::X, |writer| {
        full(writer, biases(first)[0], 0, 42);
    });
    let next = one_field(second, 0, Some(&prior), W::X, |writer| {
        full(writer, biases(second)[0], 42, 77);
    });
    assert_eq!(next.context(), second);
    assert_eq!(next.profile(), BaselineProtocol::Legacy6);
    assert_eq!(next.archival()[0].word(W::X), 77.0_f32.to_bits());
    assert_eq!(prior.context(), first);
    assert_eq!(prior.archival()[0].word(W::X), 42.0_f32.to_bits());
}

#[test]
fn unchanged_sections_retain_data_but_record_the_callers_current_context() {
    let first = legacy([1000.25, -2000.75, 3000.5]);
    let second = legacy([2000.25, -3000.75, 4000.5]);
    let prior = seed(first, &[1], 0x7fc0_1234);
    let next = parse(second, 0, &[0], Some(&prior));
    assert_eq!(next.context(), second);
    assert_eq!(next.archival(), prior.archival());
    assert_eq!(next.current(), prior.current());
    assert_eq!(
        super::SnapshotHud::zero(MODERN).profile(),
        BaselineProtocol::Cod4x21
    );
}

#[test]
fn every_hud_time_field_uses_wrapping_message_time_or_absolute_full_bits() {
    for context in [MODERN, legacy([0.0; 3])] {
        for &(word, bits) in &FIELDS {
            if bits != -97 {
                continue;
            }
            let prior = one_field(context, 7, None, word, |writer| {
                writer.write_bits(1, 2).expect("Changed relative time");
                writer.write_bits(255, 8).expect("Age");
            });
            assert_eq!(prior.archival()[0].word(word), 7_u32.wrapping_sub(255));
            let next = one_field(context, 0xffff_fffe, Some(&prior), word, |writer| {
                writer.write_bits(3, 2).expect("Changed absolute time");
                writer
                    .write_i32(0x89ab_cdef_u32.cast_signed())
                    .expect("Absolute bits");
            });
            assert_eq!(next.archival()[0].word(word), 0x89ab_cdef);
        }
    }
}
