// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn every_truncation_preserves_dirty_final_cells_and_preexisting_cached_reader_bits() {
    let input = fixture(
        &[Update::Kind(2, 0, 7), Update::Kind(4, 0, 13)],
        true,
        false,
    );
    for end in 1..input.len() {
        let mut reader = BodyReader::try_new(&input[..end], input.len()).expect("Truncated");
        assert_eq!(reader.read_bits(3), Ok(5));
        let before = position(&reader);
        let mut expected = reader.clone();
        let mut output = dirty(8);
        let original = output.clone();
        assert!(
            SnapshotEntityPlan::parse(
                &mut reader,
                EntityDecodeContext::Cod4x21,
                0,
                sources(&[], &[]),
                BUDGET,
                &mut output
            )
            .is_err()
        );
        assert_eq!(output, original);
        assert_eq!(position(&reader), before);
        assert_eq!(reader.read_bits(5), expected.read_bits(5));
    }
}

#[test]
fn wire_order_duplicate_and_late_bad_record_failures_never_publish_a_prefix() {
    for updates in [
        vec![Update::Kind(3, 0, 3), Update::Retain(3)],
        vec![Update::Kind(3, 0, 3), Update::Retain(2)],
    ] {
        let input = fixture(&updates, false, false);
        let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
        let mut output = dirty(8);
        let before = output.clone();
        assert!(
            SnapshotEntityPlan::parse(
                &mut reader,
                EntityDecodeContext::Cod4x21,
                0,
                sources(&[], &[]),
                BUDGET,
                &mut output
            )
            .is_err()
        );
        assert_eq!(position(&reader), (0, input.len(), 0));
        assert_eq!(output, before);
    }
    let mut bytes = [0; 128];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    write_update(&mut writer, Update::Kind(2, 0, 7));
    index(&mut writer, 4);
    writer.write_bits(2, 2).expect("Not removed, delta");
    writer.write_bits(0, 6).expect("Invalid changed count");
    index(&mut writer, 1023);
    let mut reader = BodyReader::try_new(writer.as_bytes(), writer.as_bytes().len()).expect("Body");
    let mut output = dirty(8);
    let before = output.clone();
    assert!(matches!(
        SnapshotEntityPlan::parse(
            &mut reader,
            EntityDecodeContext::Cod4x21,
            0,
            sources(&[], &[]),
            BUDGET,
            &mut output
        ),
        Err(EntityStreamError::Delta(_))
    ));
    assert_eq!(output, before);
    assert_eq!(reader.byte_position(), 0);
}

#[test]
fn independent_budgets_charge_all_sources_wire_terminator_and_unchanged_tail_before_publication() {
    let previous = [state(1, 1), state(9, 9)];
    let baselines = [state(2, 2), state(8, 8)];
    let input = fixture(&[Update::Retain(2)], false, false);
    for (budget, capacity, error) in [
        (
            EntityStreamBudget {
                max_previous_entities: 1,
                ..BUDGET
            },
            4,
            EntityStreamError::PreviousBudget,
        ),
        (
            EntityStreamBudget {
                max_baselines: 1,
                ..BUDGET
            },
            4,
            EntityStreamError::BaselineBudget,
        ),
        (
            EntityStreamBudget {
                max_wire_records: 1,
                ..BUDGET
            },
            4,
            EntityStreamError::RecordBudget,
        ),
        (
            EntityStreamBudget {
                max_entities: 2,
                ..BUDGET
            },
            4,
            EntityStreamError::EntityBudget,
        ),
        (BUDGET, 2, EntityStreamError::OutputCapacity),
    ] {
        let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
        let mut output = dirty(capacity);
        let before = output.clone();
        assert_eq!(
            SnapshotEntityPlan::parse(
                &mut reader,
                EntityDecodeContext::Cod4x21,
                0,
                sources(&previous, &baselines),
                budget,
                &mut output
            )
            .err(),
            Some(error)
        );
        assert_eq!(output, before);
        assert_eq!(reader.byte_position(), 0);
    }
}

#[test]
fn invalid_sources_reject_even_if_the_stream_never_mentions_them() {
    let input = fixture(&[], false, false);
    for (previous, baselines, baseline) in [
        (vec![state(3, 3), state(1, 1)], vec![], false),
        (vec![state(1, 1), state(1, 2)], vec![], false),
        (vec![state(1023, 17)], vec![], false),
        (vec![], vec![state(3, 3), state(1, 1)], true),
        (vec![], vec![state(1, 1), state(1, 2)], true),
    ] {
        let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
        let mut output = dirty(8);
        let before = output.clone();
        assert_eq!(
            SnapshotEntityPlan::parse(
                &mut reader,
                EntityDecodeContext::Cod4x21,
                0,
                sources(&previous, &baselines),
                BUDGET,
                &mut output
            )
            .err(),
            Some(EntityStreamError::Source { baseline })
        );
        assert_eq!(output, before);
        assert_eq!(reader.byte_position(), 0);
    }
}
