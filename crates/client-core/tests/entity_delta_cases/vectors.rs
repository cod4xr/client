// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn independent_physical_no_delta_and_removal_vectors_preserve_following_fields() {
    let mut stream = decoder();
    let mut reader = BodyReader::try_new(&[0xa1, 7], 2).expect("Physical number zero/no delta");
    let prepared = stream.prepare(&mut reader).expect("Index");
    assert_eq!(prepared.number(), 0);
    let state = present(prepared.decode(None, 123).expect("Zero state"));
    assert_eq!(
        (
            state.number(),
            state.word(W::Kind),
            state.word(W::PositionTime)
        ),
        (0, 0, 0)
    );
    assert_eq!(reader.read_u8(), Ok(7));
    assert_eq!(reader.read_bits(5), Ok(20));
    let mut removed = BodyReader::try_new(&[0x82, 0x10, 7], 3).expect("Physical removal 32");
    assert_eq!(
        stream
            .prepare(&mut removed)
            .expect("Later index")
            .decode(None, 0),
        Ok(EntityDelta::Removed { number: 32 })
    );
    assert_eq!(removed.read_u8(), Ok(7));
}

#[test]
fn no_delta_retains_a_complete_owned_baseline_with_source_and_prior_owner_release() {
    let mut bytes = fixture(17, None, |_| {});
    let state = {
        let original = reference(17);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let mut stream = decoder();
        let prepared = stream.prepare(&mut reader).expect("Index");
        assert_eq!(prepared.number(), original.number());
        let state = present(
            prepared
                .decode(Some(&original), u32::MAX)
                .expect("Retained whole state"),
        );
        assert_eq!(state, original);
        state
    };
    bytes.fill(0);
    assert_eq!(state.word(W::PositionX), 0x7fc0_1234);
    assert_eq!(state.word(W::Union1), 0xffff_fffd);
    assert_eq!(format!("{state:?}"), "EntityState { number: 17, .. }");
}

#[test]
fn type_changes_select_every_target_layout_and_preserve_other_nonzero_words() {
    let original = reference(17);
    for target in (0_u8..18).chain([18, 255]) {
        let count = match target {
            12 => 58,
            13 => 60,
            _ => 59,
        };
        let bytes = fixture(17, Some(count), |writer| {
            kind(writer, 1, target);
            for position in 1..count {
                if position == count - 1 {
                    writer
                        .write_bits(3, 2)
                        .expect("Final changed nonzero Part3");
                    writer
                        .write_i32(0xa55a_5aa5_u32.cast_signed())
                        .expect("Full XOR Part3");
                    continue;
                }
                // Independent native hint-2 positions; every ordinary field retains
                // its prior semantic word through an outer unchanged flag.
                match (target, position) {
                    (1, 1 | 2) => writer.write_i32(0).expect("Absolute unconditional origin"),
                    (3, 6 | 7 | 13 | 14) | (13, 1..=3 | 5 | 6) => {
                        writer.write_bits(0, 2).expect("Absolute float zero");
                    }
                    (3, 12) | (13, 7) => writer.write_bits(0, 9).expect("Relative time zero"),
                    _ => writer
                        .write_bits(0, 1)
                        .expect("Retained or unconditional zero"),
                }
            }
            writer.write_u8(7).expect("Following opcode");
            writer.write_bits(21, 5).expect("Following cached bits");
        });
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let state = present(
            decoder()
                .prepare(&mut reader)
                .expect("Index")
                .decode(Some(&original), 0)
                .expect("Target layout"),
        );
        assert_eq!(state.word(W::Kind), u32::from(target));
        assert_eq!(
            state.word(W::PositionX),
            if matches!(target, 1 | 13) {
                0
            } else {
                original.word(W::PositionX)
            }
        );
        assert_eq!(
            state.word(W::PositionY),
            if matches!(target, 1 | 13) {
                0
            } else {
                original.word(W::PositionY)
            }
        );
        assert_eq!(
            state.word(W::PositionZ),
            if target == 13 {
                0
            } else {
                original.word(W::PositionZ)
            }
        );
        assert_eq!(
            state.word(W::AngleY),
            if target == 13 {
                0
            } else {
                original.word(W::AngleY)
            }
        );
        assert_eq!(state.word(W::Union1), original.word(W::Union1));
        assert_eq!(state.word(W::Part3), 0xa55a_5aa5);
        assert_eq!(reader.read_u8(), Ok(7));
        assert_eq!(reader.read_bits(5), Ok(21));
        assert_eq!(reader.remaining_bytes(), 0);
    }
}

#[test]
fn changed_prefix_uses_signed_xor_and_absolute_coordinates_without_erasing_a_retained_tail() {
    let original = reference(1023);
    let bytes = fixture(1023, Some(4), |writer| {
        kind(writer, 1, 1);
        writer
            .write_i32(2.0_f32.to_bits().cast_signed())
            .expect("Absolute X, not XOR");
        writer.write_i32(0x7fc0_4321).expect("Raw absolute Y");
        writer.write_bits(3, 2).expect("Changed nonzero union");
        writer.write_u8(0xff).expect("Old minus three XOR new two");
    });
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    let state = present(
        decoder()
            .prepare(&mut reader)
            .expect("Number")
            .decode(Some(&original), 99)
            .expect("Complete prefix"),
    );
    assert_eq!(state.word(W::PositionX), 2.0_f32.to_bits());
    assert_eq!(state.word(W::PositionY), 0x7fc0_4321);
    assert_eq!(state.word(W::Union1), 2);
    assert_eq!(state.word(W::PositionZ), original.word(W::PositionZ));
    assert_eq!(state.word(W::AngleY), original.word(W::AngleY));
}
