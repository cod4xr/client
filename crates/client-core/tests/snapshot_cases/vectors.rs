// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, Change, SnapshotClientSlot, SnapshotEntitySlot, SnapshotPlan, body, contexts,
    empty_tail, encoded, input, output, prefix, values,
};

#[test]
fn independent_empty_snapshot_vector_consumes_exact_body_and_retains_following_fields() {
    for context in contexts() {
        let bytes = [0, 0, 0, 0, 0, 0, 1, 0, 0x80, 0xff, 0xab, 7];
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Physical body");
        let mut entities = [];
        let mut clients = [];
        let prepared = SnapshotPlan::prepare(
            &mut reader,
            input(context, 1, None),
            output(&mut entities, &mut clients),
        )
        .expect("Whole snapshot");
        assert_eq!(prepared.entity_stats().wire_records, 1);
        assert_eq!(prepared.client_stats().wire_records, 1);
        assert_eq!(prepared.consumed_bytes(), 11);
        let state = prepared.commit_transmitted().expect("Commit");
        assert_eq!(state.header().server_time(), 0);
        assert_eq!(state.consumed_bytes(), 11);
        assert_eq!(state.message().server_command_sequence(), 0xffff_fffe);
        assert_eq!(values(&state), (vec![], vec![]));
        assert_eq!(reader.byte_position(), 11);
        assert_eq!(reader.read_u8().expect("Next service"), 7);
        assert_eq!(reader.read_bits(5).expect("Cached tail"), 21);
    }
}
#[test]
fn full_and_delta_snapshot_merge_both_outputs_and_keep_their_unused_tails() {
    for context in contexts() {
        let full = body(
            0,
            true,
            &[Change::Set(1, 0, 2), Change::Set(7, 0, 9)],
            &[Change::Set(0, 0, 17), Change::Set(63, 0, 511)],
        );
        let mut reader = BodyReader::try_new(&full, full.len()).expect("Full body");
        let mut old_entities = vec![SnapshotEntitySlot::default(); 4];
        let mut old_clients = vec![SnapshotClientSlot::default(); 4];
        let old = SnapshotPlan::prepare(
            &mut reader,
            input(context, 10, None),
            output(&mut old_entities, &mut old_clients),
        )
        .expect("Full")
        .commit_transmitted()
        .expect("Commit");
        assert_eq!(
            values(&old),
            (vec![(1, 2), (7, 9)], vec![(0, 17), (63, 511)])
        );
        let delta = body(
            5,
            true,
            &[Change::Remove(1), Change::Set(5, 0, 13)],
            &[Change::Set(0, 17, 23), Change::Remove(63)],
        );
        let mut reader = BodyReader::try_new(&delta, delta.len()).expect("Delta body");
        let (mut entities, mut clients) = super::dirty();
        let saved_entities = entities.clone();
        let saved_clients = clients.clone();
        let next = SnapshotPlan::prepare(
            &mut reader,
            input(context, 15, Some(old.as_source())),
            output(&mut entities, &mut clients),
        )
        .expect("Exact message distance")
        .commit_transmitted()
        .expect("Delta commit");
        assert_eq!(values(&next), (vec![(5, 13), (7, 9)], vec![(0, 23)]));
        assert_eq!(next.as_source().message().sequence(), 15);
        assert_eq!(next.as_source().profile(), context.profile());
        assert_eq!(&entities[2..], &saved_entities[2..]);
        assert_eq!(&clients[1..], &saved_clients[1..]);
    }
}
#[test]
fn whole_snapshot_can_start_in_existing_bit_cache_without_aligning_sections() {
    for context in contexts() {
        let bytes = encoded(|writer| {
            writer.write_bits(5, 3).expect("Leading bits");
            prefix(writer, 0, true);
            empty_tail(writer);
            writer.write_u8(7).expect("Following service");
        });
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        assert_eq!(reader.read_bits(3).expect("Prefix"), 5);
        let mut entities = [];
        let mut clients = [];
        SnapshotPlan::prepare(
            &mut reader,
            input(context, 1, None),
            output(&mut entities, &mut clients),
        )
        .expect("Cached snapshot")
        .commit_transmitted()
        .expect("Commit");
        assert_eq!(reader.read_u8().expect("Following service"), 7);
    }
}

#[test]
fn player_stats_entities_and_clients_commit_together_and_survive_as_one_delta_source() {
    for context in contexts() {
        let bytes = encoded(|writer| {
            writer.write_i32(-1).expect("Time bits");
            writer.write_u8(0).expect("Full");
            writer.write_u8(0xa5).expect("Flags");
            writer.write_bits(1, 1).expect("Transmitted");
            writer.write_bits(0, 8).expect("No scalar fields");
            writer.write_bits(1, 1).expect("Changed stats");
            writer.write_bits(1, 5).expect("Stat zero mask");
            writer.write_i16(-123).expect("Signed stat");
            writer.write_bits(0, 12).expect("Remaining player arrays");
            super::entity(writer, Change::Set(1, 0, 13));
            writer.write_bits(0xffe, 12).expect("Entity terminator");
            super::client(writer, Change::Set(63, 0, 511));
            writer.write_bits(0, 1).expect("Client end");
        });
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let (mut old_entities, mut old_clients) = super::dirty();
        let old = SnapshotPlan::prepare(
            &mut reader,
            input(context, 10, None),
            output(&mut old_entities, &mut old_clients),
        )
        .expect("All parts")
        .commit_transmitted()
        .expect("Commit");
        assert_eq!(old.player().stats().get(0), Some(-123));
        assert_eq!(values(&old), (vec![(1, 13)], vec![(63, 511)]));
        let delta = body(1, true, &[], &[]);
        let mut reader = BodyReader::try_new(&delta, delta.len()).expect("Delta body");
        let (mut entities, mut clients) = super::dirty();
        let next = SnapshotPlan::prepare(
            &mut reader,
            input(context, 11, Some(old.as_source())),
            output(&mut entities, &mut clients),
        )
        .expect("Coherent delta")
        .commit_transmitted()
        .expect("Commit");
        assert_eq!(next.player().stats(), old.player().stats());
        assert_eq!(values(&next), values(&old));
    }
}
