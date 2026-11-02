// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn successful_stream_uses_existing_cached_bits_without_aligning_or_consuming_following_data() {
    let input = fixture(&[Update::Kind(2, 0, 7), Update::Kind(4, 0, 13)], true, true);
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Body with prefix");
    assert_eq!(reader.read_bits(3), Ok(5));
    let start = reader.byte_position();
    let mut output = dirty(8);
    let plan = SnapshotEntityPlan::parse(
        &mut reader,
        EntityDecodeContext::Cod4x21,
        0,
        sources(&[], &[]),
        BUDGET,
        &mut output,
    )
    .expect("Complete stream with cached input");
    assert_eq!(values(&plan), [(2, 7), (4, 13)]);
    assert_eq!(plan.consumed_bytes(), reader.byte_position() - start);
    assert_eq!(reader.read_u8(), Ok(7));
    assert_eq!(reader.read_bits(5), Ok(21));
    assert_eq!(reader.remaining_bytes(), 0);
}

#[test]
fn independent_physical_terminator_keeps_following_byte_and_cached_bits_distinct() {
    for input in [b"\xfe\x0f".as_slice(), &[0xfe, 0x5f, 7, 1]] {
        let mut reader = BodyReader::try_new(input, input.len()).expect("Physical marker");
        let mut output = dirty(2);
        let before = output.clone();
        let plan = SnapshotEntityPlan::parse(
            &mut reader,
            EntityDecodeContext::Cod4x21,
            0,
            sources(&[], &[]),
            BUDGET,
            &mut output,
        )
        .expect("Exact reserved index");
        assert!(plan.entities().next().is_none());
        assert_eq!(
            (
                plan.stats().wire_records,
                plan.stats().updates,
                plan.consumed_bytes()
            ),
            (1, 0, 2)
        );
        if input.len() > 2 {
            assert_eq!(reader.read_u8(), Ok(7));
            assert_eq!(reader.read_bits(5), Ok(21));
        }
        assert_eq!(output, before);
    }
}

#[test]
fn sparse_merge_retains_unmentioned_values_uses_baselines_and_ignores_unknown_removals() {
    let previous = [
        state(1, 1),
        state(3, 3),
        state(5, 5),
        state(9, 9),
        state(11, 11),
    ];
    let baselines = [state(2, 2), state(4, 4), state(8, 8), state(1023, 17)];
    let input = fixture(
        &[
            Update::Kind(2, 2, 12),
            Update::Retain(3),
            Update::Remove(5),
            Update::Remove(6),
            Update::Retain(8),
            Update::Kind(9, 9, 0),
        ],
        false,
        true,
    );
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
    let mut output = dirty(10);
    let tail = output[6..].to_vec();
    let plan = SnapshotEntityPlan::parse(
        &mut reader,
        EntityDecodeContext::Cod4x21,
        123,
        sources(&previous, &baselines),
        BUDGET,
        &mut output,
    )
    .expect("Complete merged stream");
    assert_eq!(
        values(&plan),
        [(1, 1), (2, 12), (3, 3), (8, 8), (9, 0), (11, 11)]
    );
    let summary = plan.stats();
    assert_eq!(
        (summary.wire_records, summary.updates, summary.removals),
        (7, 6, 2)
    );
    assert_eq!(
        (
            summary.unchanged_entities,
            summary.baseline_references,
            summary.zero_references,
            summary.output_entities
        ),
        (2, 2, 1, 6)
    );
    assert_eq!(reader.read_u8(), Ok(7));
    assert_eq!(reader.read_bits(5), Ok(21));
    assert_eq!(&output[6..], tail);
}

#[test]
fn missing_baselines_use_zero_and_previous_state_takes_precedence_over_matching_baselines() {
    let previous = [state(3, 7)];
    let baselines = [state(3, 9)];
    let input = fixture(&[Update::Retain(2), Update::Retain(3)], false, false);
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
    let mut output = dirty(4);
    let plan = SnapshotEntityPlan::parse(
        &mut reader,
        EntityDecodeContext::Cod4x21,
        0,
        sources(&previous, &baselines),
        BUDGET,
        &mut output,
    )
    .expect("Source precedence");
    assert_eq!(values(&plan), [(2, 0), (3, 7)]);
    assert_eq!(
        (
            plan.stats().zero_references,
            plan.stats().baseline_references
        ),
        (1, 0)
    );
}

#[test]
fn maximum_number_uses_the_increment_shortcut_for_terminator_without_a_removal_bit() {
    let previous = [state(1022, 17)];
    let mut bytes = [0; 32];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    write_update(&mut writer, Update::Retain(1022));
    writer
        .write_bits(1, 1)
        .expect("Next number 1023 ends before body");
    writer.write_u8(7).expect("Following byte");
    let mut reader = BodyReader::try_new(writer.as_bytes(), writer.as_bytes().len()).expect("Body");
    let mut output = dirty(2);
    let plan = SnapshotEntityPlan::parse(
        &mut reader,
        EntityDecodeContext::Cod4x21,
        0,
        sources(&previous, &[]),
        BUDGET,
        &mut output,
    )
    .expect("Terminal shortcut");
    assert_eq!(values(&plan), [(1022, 17)]);
    assert_eq!(plan.stats().wire_records, 2);
    assert_eq!(reader.read_u8(), Ok(7));
}

#[test]
fn explicit_legacy_context_decodes_nonzero_origins_within_the_same_complete_merge() {
    let previous = [state(1, 1)];
    let mut bytes = [0; 128];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    index(&mut writer, 1);
    writer.write_bits(0, 1).expect("Not removed");
    writer.write_bits(1, 1).expect("Delta");
    writer.write_bits(3, 6).expect("Through unconditional Y");
    writer.write_bits(0, 1).expect("Retained type one");
    for value in [65, 63] {
        writer.write_bits(0, 1).expect("Small legacy origin");
        writer.write_bits(value, 7).expect("Relative origin bits");
    }
    index(&mut writer, 1023);
    let mut reader = BodyReader::try_new(writer.as_bytes(), writer.as_bytes().len()).expect("Body");
    let mut output = dirty(2);
    let context = EntityDecodeContext::Legacy6(LegacyMapCenter::new([1000.0; 3]).expect("Context"));
    let plan = SnapshotEntityPlan::parse(
        &mut reader,
        context,
        0,
        sources(&previous, &[]),
        BUDGET,
        &mut output,
    )
    .expect("Legacy merge");
    let result = plan.entities().next().expect("One entity");
    assert_eq!(plan.profile(), BaselineProtocol::Legacy6);
    assert_eq!(result.word(W::PositionX), 1.0_f32.to_bits());
    assert_eq!(result.word(W::PositionY), (-1.0_f32).to_bits());
}
