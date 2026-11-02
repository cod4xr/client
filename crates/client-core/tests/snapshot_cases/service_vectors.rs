// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, Change, body, contexts, dirty, empty_tail, encoded, input, output, prefix, sample,
    values,
};
use cod4xr_client_core::{
    service_controls::ServiceControlBudget, snapshots::services::SnapshotServicePlan,
};

#[test]
fn physical_snapshot_service_carrier_stops_at_eof_and_preserves_padding_and_cached_bits() {
    for context in contexts() {
        let bytes = [0, 6, 0, 0, 0, 0, 0, 0, 1, 0, 0x80, 0xff, 0xab, 0, 7, 4, 11];
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Physical carrier");
        let mut entities = [];
        let mut clients = [];
        let prepared = SnapshotServicePlan::prepare(
            &mut reader,
            input(context, 1, None),
            ServiceControlBudget { max_records: 4 },
            output(&mut entities, &mut clients),
        )
        .expect("Complete EOF carrier");
        assert_eq!(prepared.record_count(), 4);
        assert_eq!(prepared.consumed_bytes(), 15);
        assert_eq!(prepared.header().profile(), context.profile());
        assert_eq!(prepared.prediction_request(), None);
        let plan = prepared.commit_transmitted().expect("Commit");
        assert_eq!(plan.record_count(), 4);
        assert_eq!(plan.consumed_bytes(), 15);
        assert_eq!(plan.snapshot().consumed_bytes(), 11);
        assert_eq!(reader.byte_position(), 15);
        assert_eq!(reader.read_u8().expect("Uninterpreted padding"), 4);
        assert_eq!(reader.read_bits(5).expect("Cached tail"), 21);
    }
}
#[test]
fn service_and_header_byte_operations_preserve_existing_cache_without_alignment() {
    for context in contexts() {
        let bytes = encoded(|writer| {
            writer.write_bits(5, 3).expect("Existing cache");
            writer.write_u8(0).expect("NOP");
            writer.write_u8(6).expect("Snapshot");
            prefix(writer, 0, true);
            empty_tail(writer);
            writer.write_u8(0).expect("NOP");
            writer.write_u8(7).expect("EOF");
            writer.write_u8(0xff).expect("Padding");
        });
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Carrier");
        assert_eq!(reader.read_bits(3).expect("Cache"), 5);
        let start = reader.byte_position();
        let mut entities = [];
        let mut clients = [];
        let plan = SnapshotServicePlan::prepare(
            &mut reader,
            input(context, 1, None),
            ServiceControlBudget::default(),
            output(&mut entities, &mut clients),
        )
        .expect("Cached carrier")
        .commit_transmitted()
        .expect("Commit");
        assert_eq!(reader.byte_position() - start, plan.consumed_bytes());
        assert_eq!(plan.snapshot().consumed_bytes(), 10);
        assert_eq!(reader.read_u8().expect("Padding"), 0xff);
        assert_eq!(reader.remaining_cached_bits(), 2);
    }
}
#[test]
fn committed_service_plan_outlives_carrier_and_provides_coherent_cooked_references() {
    for context in contexts() {
        let (mut entities, mut clients) = dirty();
        let plan = {
            let mut bytes = vec![6];
            bytes.extend(body(
                0,
                false,
                &[Change::Set(1, 0, 13)],
                &[Change::Set(63, 0, 511)],
            ));
            bytes.push(7);
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Carrier");
            let prepared = SnapshotServicePlan::prepare(
                &mut reader,
                input(context, 1, None),
                ServiceControlBudget::default(),
                output(&mut entities, &mut clients),
            )
            .expect("Services");
            assert_eq!(
                prepared
                    .prediction_request()
                    .expect("Request")
                    .command_time(),
                0
            );
            prepared
                .commit_with_prediction(&sample(context.profile()))
                .expect("Sample")
        };
        assert_eq!(plan.record_count(), 2);
        assert_eq!(values(plan.snapshot()), (vec![(1, 13)], vec![(63, 511)]));
        assert_eq!(plan.snapshot().as_source().message().sequence(), 1);
    }
}
