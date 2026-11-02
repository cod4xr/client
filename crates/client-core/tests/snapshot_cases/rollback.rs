// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, Change, ClientStreamError, EntityStreamError, SnapshotError, SnapshotPlan, body,
    contexts, dirty, encoded, entity, input, output, position, prefix,
};
use cod4xr_client_core::client_states::ClientStateError;

#[test]
fn every_truncated_complete_snapshot_preserves_both_output_buffers_and_cached_reader() {
    for context in contexts() {
        let bytes = encoded(|writer| {
            writer.write_bits(5, 3).expect("Prefix");
            prefix(writer, 0, true);
            entity(writer, Change::Set(1, 0, 13));
            writer.write_bits(0xffe, 12).expect("Entity terminator");
            super::client(writer, Change::Set(63, 0, 511));
            writer.write_bits(0, 1).expect("Client end");
        });
        for length in 1..bytes.len() {
            let mut reader = BodyReader::try_new(&bytes[..length], length).expect("Truncated body");
            assert_eq!(reader.read_bits(3).expect("Prefix"), 5);
            let before = position(&reader);
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            assert!(
                SnapshotPlan::prepare(
                    &mut reader,
                    input(context, 1, None),
                    output(&mut entities, &mut clients)
                )
                .is_err(),
                "prefix {length}"
            );
            assert_eq!(position(&reader), before);
            assert_eq!((entities, clients), saved);
        }
    }
}
#[test]
fn invalid_last_client_prefix_cancels_valid_player_and_entity_decoding() {
    for context in contexts() {
        let bytes = encoded(|writer| {
            prefix(writer, 0, true);
            entity(writer, Change::Set(1, 0, 13));
            writer.write_bits(0xffe, 12).expect("Entity terminator");
            writer.write_bits(1 | (63 << 2), 8).expect("Client 63");
            writer.write_bits(2, 2).expect("Not removed, has delta");
            writer.write_bits(31, 5).expect("Invalid client prefix");
        });
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let (mut entities, mut clients) = dirty();
        let saved = (entities.clone(), clients.clone());
        assert_eq!(
            SnapshotPlan::prepare(
                &mut reader,
                input(context, 1, None),
                output(&mut entities, &mut clients)
            )
            .expect_err("Late failure"),
            SnapshotError::Clients(ClientStreamError::Delta(ClientStateError::FieldCount))
        );
        assert_eq!(reader.byte_position(), 0);
        assert_eq!((entities, clients), saved);
    }
}
#[test]
fn independent_stream_budgets_and_capacity_failures_never_publish_the_other_output() {
    for context in contexts() {
        for case in 0..6 {
            let bytes = body(
                0,
                true,
                &[Change::Set(1, 0, 13)],
                &[Change::Set(63, 0, 511)],
            );
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            let mut selected = input(context, 1, None);
            let expected = match case {
                0 => {
                    selected.budget.entities.max_wire_records = 1;
                    SnapshotError::Entities(EntityStreamError::RecordBudget)
                }
                1 => {
                    selected.budget.entities.max_entities = 0;
                    SnapshotError::Entities(EntityStreamError::EntityBudget)
                }
                2 => SnapshotError::Entities(EntityStreamError::OutputCapacity),
                3 => {
                    selected.budget.clients.max_wire_records = 1;
                    SnapshotError::Clients(ClientStreamError::RecordBudget)
                }
                4 => {
                    selected.budget.clients.max_clients = 0;
                    SnapshotError::Clients(ClientStreamError::ClientBudget)
                }
                _ => SnapshotError::Clients(ClientStreamError::OutputCapacity),
            };
            let entity_capacity = if case == 2 { 0 } else { entities.len() };
            let client_capacity = if case == 5 { 0 } else { clients.len() };
            assert_eq!(
                SnapshotPlan::prepare(
                    &mut reader,
                    selected,
                    output(
                        &mut entities[..entity_capacity],
                        &mut clients[..client_capacity]
                    )
                )
                .expect_err("Policy failure"),
                expected
            );
            assert_eq!(reader.byte_position(), 0);
            assert_eq!((entities, clients), saved);
        }
    }
}
#[test]
fn dropping_validated_snapshot_preserves_outputs_and_allows_the_same_input_retry() {
    for context in contexts() {
        let bytes = body(
            0,
            true,
            &[Change::Set(1, 0, 13)],
            &[Change::Set(63, 0, 511)],
        );
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let (mut entities, mut clients) = dirty();
        let saved = (entities.clone(), clients.clone());
        let prepared = SnapshotPlan::prepare(
            &mut reader,
            input(context, 1, None),
            output(&mut entities, &mut clients),
        )
        .expect("Prepared");
        drop(prepared);
        assert_eq!(reader.byte_position(), 0);
        assert_eq!((&entities, &clients), (&saved.0, &saved.1));
        SnapshotPlan::prepare(
            &mut reader,
            input(context, 1, None),
            output(&mut entities, &mut clients),
        )
        .expect("Retry")
        .commit_transmitted()
        .expect("Commit");
    }
}
