// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BUDGET, BodyReader, ClientStreamBudget, PROFILES, SnapshotClientPlan, SnapshotClientSlot,
    Update, dirty, encoded, fixture, source, state, values,
};

#[test]
fn independent_end_bit_and_client_sixty_three_vectors_keep_following_data() {
    for profile in PROFILES {
        // 0x16: prior 3 bits=6, zero stream-end bit, following cached 1 bit=1.
        let mut reader = BodyReader::try_new(&[0x16, 0xa7], 2).expect("Physical vector");
        assert_eq!(reader.read_bits(3).expect("Prefix"), 6);
        let mut output = [];
        let plan =
            SnapshotClientPlan::parse(&mut reader, profile, source(&[]), BUDGET, &mut output)
                .expect("Empty stream");
        assert!(values(&plan).is_empty());
        assert_eq!(plan.consumed_bytes(), 0);
        assert_eq!(plan.stats().wire_records, 1);
        assert_eq!(reader.read_u8().expect("Following byte"), 0xa7);
        assert_eq!(reader.read_bits(1).expect("Following bit"), 1);
        let mut reader = BodyReader::try_new(&[0xfd, 0], 2).expect("Client 63 vector");
        let mut output = [SnapshotClientSlot::default()];
        let plan =
            SnapshotClientPlan::parse(&mut reader, profile, source(&[]), BUDGET, &mut output)
                .expect("63 then end");
        assert_eq!(values(&plan), [(63, 0)]);
        assert_eq!(plan.stats().wire_records, 2);
        assert_eq!(plan.stats().zero_references, 1);
    }
}

#[test]
fn sparse_merge_keeps_prior_prefix_tail_and_uses_prior_or_zero_reference() {
    let previous = [
        state(1, 101),
        state(3, 303),
        state(5, 505),
        state(9, 99),
        state(63, 163),
    ];
    let bytes = fixture(
        &[
            Update::Model(3, 303, 111),
            Update::Remove(5),
            Update::Remove(6),
            Update::Retain(7),
            Update::Model(8, 0, 511),
        ],
        false,
        false,
    );
    for profile in PROFILES {
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let mut output = dirty(9);
        let saved = output.clone();
        let plan =
            SnapshotClientPlan::parse(&mut reader, profile, source(&previous), BUDGET, &mut output)
                .expect("Merged stream");
        assert_eq!(
            values(&plan),
            [(1, 101), (3, 111), (7, 0), (8, 511), (9, 99), (63, 163)]
        );
        let stats = plan.stats();
        assert_eq!(stats.wire_records, 6);
        assert_eq!(stats.updates, 5);
        assert_eq!(stats.removals, 2);
        assert_eq!(stats.unchanged_clients, 3);
        assert_eq!(stats.zero_references, 3);
        assert_eq!(stats.output_clients, 6);
        assert_eq!(&output[6..], &saved[6..]);
    }
    assert_eq!(previous[1].word(super::W::Model), 303);
}

#[test]
fn existing_cached_bits_and_following_fields_survive_complete_stream_merge() {
    let previous = [state(7, 341)];
    let bytes = fixture(&[Update::Model(7, 341, 17), Update::Retain(63)], true, true);
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    assert_eq!(reader.read_bits(3).expect("Prefix"), 5);
    let mut output = dirty(2);
    let plan = SnapshotClientPlan::parse(
        &mut reader,
        PROFILES[1],
        source(&previous),
        BUDGET,
        &mut output,
    )
    .expect("Merged stream");
    assert_eq!(values(&plan), [(7, 17), (63, 0)]);
    assert_eq!(reader.read_u8().expect("Following byte"), 0xa7);
    assert_eq!(reader.read_bits(2).expect("Following bits"), 3);
}

#[test]
fn all_sixty_four_clients_and_end_charge_exactly_sixty_five_headers() {
    let bytes = encoded(|writer| {
        for _ in 0..64 {
            writer
                .write_bits(3, 4)
                .expect("More increment present no delta");
        }
        writer.write_bits(0, 1).expect("Separate end");
    });
    let mut output = vec![SnapshotClientSlot::default(); 64];
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    let plan =
        SnapshotClientPlan::parse(&mut reader, PROFILES[1], source(&[]), BUDGET, &mut output)
            .expect("All clients");
    assert_eq!(plan.stats().wire_records, 65);
    assert_eq!(plan.stats().output_clients, 64);
    assert_eq!(values(&plan).last(), Some(&(63, 0)));
    // One end bit alone can retain the entire old set with no explicit updates.
    let prior = plan.as_source();
    let mut next_output = vec![SnapshotClientSlot::default(); 64];
    let mut next_reader = BodyReader::try_new(&[0], 1).expect("Empty update stream");
    let next = SnapshotClientPlan::parse(
        &mut next_reader,
        PROFILES[1],
        prior,
        ClientStreamBudget {
            max_wire_records: 1,
            ..BUDGET
        },
        &mut next_output,
    )
    .expect("Retained old set");
    assert_eq!(values(&next), values(&plan));
    assert_eq!(next.stats().updates, 0);
    assert_eq!(next.stats().unchanged_clients, 64);
}
