// SPDX-License-Identifier: AGPL-3.0-or-later
use super::legacy_entity_support::coordinate;
use super::*;

#[test]
fn every_legacy_target_layout_retains_nonzero_origins_or_applies_its_native_hint_two_forms() {
    let prior = reference(17);
    for target in (0_u8..18).chain([18, 255]) {
        let count = match target {
            12 => 58,
            13 => 60,
            _ => 59,
        };
        let mut bytes = [0; 512];
        let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
        header(&mut writer, 17, count, target);
        for position in 1..count {
            if position == count - 1 {
                writer.write_bits(3, 2).expect("Changed final Part3");
                writer
                    .write_i32(0xa55a_5aa5_u32.cast_signed())
                    .expect("Full XOR Part3");
                continue;
            }
            match (target, position) {
                (1, 1 | 2) => coordinate(&mut writer, Coordinate::Small(64)),
                (3, 6 | 7 | 13 | 14) | (13, 1..=3 | 5 | 6) => {
                    writer
                        .write_bits(0, 2)
                        .expect("Absolute unconditional float zero");
                }
                (3, 12) | (13, 7) => writer.write_bits(0, 9).expect("Unconditional message time"),
                _ => writer
                    .write_bits(0, 1)
                    .expect("Retained or unconditional zero"),
            }
        }
        writer.write_u8(7).expect("Following opcode");
        writer.write_bits(21, 5).expect("Following cached bits");
        let mut reader =
            BodyReader::try_new(writer.as_bytes(), writer.as_bytes().len()).expect("Body");
        let state = present(
            decoder([1000.0, -1000.0, 0.0])
                .prepare(&mut reader)
                .expect("Index")
                .decode(Some(&prior), 37)
                .expect("Complete target layout"),
        );
        assert_eq!(state.word(W::Kind), u32::from(target));
        for word in [W::PositionX, W::PositionY, W::PositionZ] {
            assert_eq!(
                state.word(word),
                if target == 13 { 0 } else { prior.word(word) }
            );
        }
        assert_eq!(state.word(W::Part3), 0xa55a_5aa5);
        assert_eq!(state.word(W::AngleTime), if target == 3 { 37 } else { 0 });
        assert_eq!(
            state.word(W::PositionTime),
            if target == 13 { 37 } else { 0 }
        );
        assert_eq!(reader.read_u8(), Ok(7));
        assert_eq!(reader.read_bits(5), Ok(21));
    }
}
