// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BUDGET, BodyReader, ClientStreamBudget, ClientStreamError as E, PROFILES, SnapshotClientPlan,
    Update, dirty, encoded, failure, fixture, position, source, state, write_update,
};

#[test]
fn independent_budgets_cover_unused_prior_entries_end_header_and_retained_tail() {
    let previous = [state(1, 1), state(3, 3), state(5, 5)];
    let empty = fixture(&[], false, false);
    failure(
        &empty,
        &previous,
        ClientStreamBudget {
            max_previous_clients: 2,
            ..BUDGET
        },
        3,
        E::PreviousBudget,
    );
    failure(
        &empty,
        &[],
        ClientStreamBudget {
            max_wire_records: 0,
            ..BUDGET
        },
        0,
        E::RecordBudget,
    );
    let bytes = fixture(&[Update::Remove(1)], false, false);
    failure(
        &bytes,
        &previous,
        ClientStreamBudget {
            max_wire_records: 1,
            ..BUDGET
        },
        3,
        E::RecordBudget,
    );
    failure(
        &bytes,
        &previous,
        ClientStreamBudget {
            max_clients: 1,
            ..BUDGET
        },
        3,
        E::ClientBudget,
    );
    failure(&bytes, &previous, BUDGET, 1, E::OutputCapacity);
    // Empty previous/output is valid with no retention allowance, but needs an end.
    let mut reader = BodyReader::try_new(&empty, empty.len()).expect("End");
    let mut output = [];
    assert!(
        SnapshotClientPlan::parse(
            &mut reader,
            PROFILES[1],
            source(&[]),
            ClientStreamBudget {
                max_previous_clients: 0,
                max_clients: 0,
                max_wire_records: 1
            },
            &mut output
        )
        .is_ok()
    );
}

#[test]
fn every_truncated_record_prefix_preserves_dirty_output_and_existing_cached_bits() {
    let previous = [state(17, 341), state(40, 40)];
    let bytes = fixture(
        &[
            Update::Model(1, 0, 511),
            Update::Model(17, 341, 17),
            Update::Model(63, 0, 511),
        ],
        true,
        false,
    );
    for length in 1..bytes.len() {
        let mut reader = BodyReader::try_new(&bytes[..length], length).expect("Truncation");
        assert_eq!(reader.read_bits(3).expect("Cached prefix"), 5);
        let before = position(&reader);
        let mut output = dirty(7);
        let saved = output.clone();
        assert!(
            SnapshotClientPlan::parse(
                &mut reader,
                PROFILES[1],
                source(&previous),
                BUDGET,
                &mut output
            )
            .is_err(),
            "length {length}"
        );
        assert_eq!(position(&reader), before);
        assert_eq!(output, saved);
    }
}

#[test]
fn duplicate_descending_or_late_bad_records_never_publish_a_valid_prefix() {
    for updates in [
        vec![Update::Retain(7), Update::Retain(7)],
        vec![Update::Retain(7), Update::Retain(3)],
    ] {
        let bytes = fixture(&updates, false, false);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let before = position(&reader);
        let mut output = dirty(4);
        let saved = output.clone();
        assert!(
            SnapshotClientPlan::parse(&mut reader, PROFILES[1], source(&[]), BUDGET, &mut output)
                .is_err()
        );
        assert_eq!(position(&reader), before);
        assert_eq!(output, saved);
    }
    let bytes = encoded(|writer| {
        write_update(writer, Update::Model(1, 0, 77));
        writer
            .write_bits(1 | (7 << 2), 8)
            .expect("Late record header");
        writer.write_bits(0, 1).expect("Not removed");
        writer.write_bits(1, 1).expect("Delta");
        writer.write_bits(31, 5).expect("Invalid count");
    });
    failure(
        &bytes,
        &[],
        BUDGET,
        4,
        E::Delta(super::ClientStateError::FieldCount),
    );
}

#[test]
fn unordered_and_duplicate_prior_sources_reject_even_without_explicit_updates() {
    let bytes = fixture(&[], false, false);
    for previous in [
        vec![state(7, 7), state(3, 3)],
        vec![state(3, 3), state(3, 4)],
    ] {
        failure(&bytes, &previous, BUDGET, 3, E::Source);
    }
}

#[test]
fn missing_end_bit_at_exact_byte_boundary_does_not_publish_sixty_four_records() {
    let bytes = encoded(|writer| {
        for _ in 0..64 {
            writer.write_bits(3, 4).expect("Four bits per record");
        }
    });
    assert_eq!(bytes.len(), 32);
    let mut reader = BodyReader::try_new(&bytes, 32).expect("No end bit");
    let before = position(&reader);
    let mut output = dirty(64);
    let saved = output.clone();
    assert!(
        SnapshotClientPlan::parse(&mut reader, PROFILES[1], source(&[]), BUDGET, &mut output)
            .is_err()
    );
    assert_eq!(position(&reader), before);
    assert_eq!(output, saved);
}
