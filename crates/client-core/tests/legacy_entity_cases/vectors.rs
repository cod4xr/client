// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn every_small_legacy_coordinate_value_is_exact_on_each_axis_with_inert_following_fields() {
    for value in 0..128_u8 {
        let source = fixture(17, [Coordinate::Small(value); 3], false, true);
        let mut reader = BodyReader::try_new(&source, source.len()).expect("Body");
        let baseline = decoder([1000.0, -1000.0, 0.0])
            .parse(&mut reader)
            .expect("All axes");
        let expected = f32::from(i16::from(value) - 64).to_bits();
        for word in [W::PositionX, W::PositionY, W::PositionZ] {
            assert_eq!(baseline.word(word), expected);
        }
        assert_eq!(reader.read_u8(), Ok(7));
        assert_eq!(reader.read_bits(5), Ok(21));
    }
}

#[test]
fn full_legacy_coordinate_window_preserves_native_axis_biases_and_owned_results() {
    for (center, expected) in [
        ([1000.0, -1000.0, 0.0], [1999.0_f32, -1999.0, -1.0]),
        ([1000.75, -1000.75, 0.0], [2001.0, -2001.0, -1.0]),
        ([8_388_609.0, 8_388_608.0, -0.0], [1.0, -1.0, -1.0]),
    ] {
        let mut source = fixture(1023, [Coordinate::Full(0xffff); 3], false, true);
        let baseline = {
            let mut reader = BodyReader::try_new(&source, source.len()).expect("Body");
            let baseline = decoder(center)
                .parse(&mut reader)
                .expect("Full native window");
            assert_eq!(reader.read_u8(), Ok(7));
            assert_eq!(reader.read_bits(5), Ok(21));
            baseline
        };
        source.fill(0);
        assert_eq!(baseline.number(), 1023);
        for (word, value) in [W::PositionX, W::PositionY, W::PositionZ]
            .into_iter()
            .zip(expected)
        {
            assert_eq!(baseline.word(word), value.to_bits());
        }
    }
}

#[test]
fn all_eighteen_legacy_layouts_keep_hint_two_value_encodings_distinct_from_coordinate_forms() {
    for kind in (0_u8..18).chain([18, 255]) {
        let count = match kind {
            12 => 58,
            13 => 60,
            _ => 59,
        };
        let mut bytes = [0; 512];
        let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
        header(&mut writer, 17, count, kind);
        for position in 1..count {
            match (kind, position) {
                (1, 1) => coordinate(&mut writer, Coordinate::Small(127)),
                (1, 2) => coordinate(&mut writer, Coordinate::Small(0)),
                (3, 6 | 7 | 13 | 14) | (13, 1..=3 | 5 | 6) => {
                    writer.write_bits(0, 2).expect("Unconditional float zero");
                }
                (3, 12) | (13, 7) => writer
                    .write_bits(0, 9)
                    .expect("Unconditional relative time zero"),
                _ => writer
                    .write_bits(0, 1)
                    .expect("Unchanged or unconditional integer zero"),
            }
        }
        writer.write_u8(7).expect("Following byte");
        writer.write_bits(21, 5).expect("Following bits");
        let mut reader =
            BodyReader::try_new(writer.as_bytes(), writer.as_bytes().len()).expect("Body");
        let baseline = decoder([0.0; 3])
            .parse(&mut reader)
            .expect("Native target layout");
        assert_eq!(baseline.word(W::Kind), u32::from(kind));
        assert_eq!(
            baseline.word(W::PositionX),
            if kind == 1 { 63.0_f32.to_bits() } else { 0 }
        );
        assert_eq!(
            baseline.word(W::PositionY),
            if kind == 1 { (-64.0_f32).to_bits() } else { 0 }
        );
        assert_eq!(baseline.word(W::PositionZ), 0);
        assert_eq!(reader.read_u8(), Ok(7));
        assert_eq!(reader.read_bits(5), Ok(21));
        assert_eq!(reader.remaining_bytes(), 0);
    }
}
