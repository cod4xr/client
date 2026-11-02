// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, Change, PlayerStateError, PredictionSample, SnapshotError, body, contexts, dirty,
    input, output, position, sample,
};
use cod4xr_client_core::{
    service_controls::ServiceControlBudget,
    snapshots::services::{SnapshotServiceError, SnapshotServicePlan},
};

fn carrier(transmitted: bool) -> Vec<u8> {
    let mut bytes = vec![0, 6];
    bytes.extend(body(
        0,
        transmitted,
        &[Change::Set(1, 0, 13)],
        &[Change::Set(63, 0, 511)],
    ));
    bytes.extend([0, 7]);
    bytes
}
#[test]
fn every_truncated_service_carrier_preserves_entire_outputs_and_original_reader() {
    for context in contexts() {
        let bytes = carrier(true);
        for length in 0..bytes.len() {
            let mut reader =
                BodyReader::try_new(&bytes[..length], length).expect("Truncated carrier");
            let before = position(&reader);
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            assert!(
                SnapshotServicePlan::prepare(
                    &mut reader,
                    input(context, 1, None),
                    ServiceControlBudget::default(),
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
fn unsupported_tail_services_never_skip_or_publish_valid_snapshot_parts() {
    for context in contexts() {
        for opcode in [1, 4, 5, 6, 11, 255] {
            let mut bytes = carrier(true);
            let last = bytes.len() - 1;
            bytes[last] = opcode;
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Carrier");
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            assert_eq!(
                SnapshotServicePlan::prepare(
                    &mut reader,
                    input(context, 1, None),
                    ServiceControlBudget::default(),
                    output(&mut entities, &mut clients)
                )
                .expect_err("Unsupported tail"),
                SnapshotServiceError::UnsupportedService(opcode)
            );
            assert_eq!(reader.byte_position(), 0);
            assert_eq!((entities, clients), saved);
        }
    }
}
#[test]
fn eof_without_snapshot_and_unsupported_prefixes_reject_without_publication() {
    for context in contexts() {
        for (bytes, expected) in [
            (vec![0, 7], SnapshotServiceError::MissingSnapshot),
            (vec![0, 4, 7], SnapshotServiceError::UnsupportedService(4)),
            (vec![0, 11, 7], SnapshotServiceError::UnsupportedService(11)),
            (vec![0], SnapshotServiceError::MissingEof),
        ] {
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Carrier");
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            assert_eq!(
                SnapshotServicePlan::prepare(
                    &mut reader,
                    input(context, 1, None),
                    ServiceControlBudget::default(),
                    output(&mut entities, &mut clients)
                )
                .expect_err("Invalid prefix"),
                expected
            );
            assert_eq!(reader.byte_position(), 0);
            assert_eq!((entities, clients), saved);
        }
    }
}
#[test]
fn service_budget_counts_prefix_snapshot_tail_and_eof_and_allows_valid_retry() {
    for context in contexts() {
        let bytes = carrier(true);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Carrier");
        let (mut entities, mut clients) = dirty();
        let saved = (entities.clone(), clients.clone());
        for maximum in 0..4 {
            let expected = if maximum == 0 {
                SnapshotServiceError::InvalidBudget
            } else {
                SnapshotServiceError::RecordBudget
            };
            assert_eq!(
                SnapshotServicePlan::prepare(
                    &mut reader,
                    input(context, 1, None),
                    ServiceControlBudget {
                        max_records: maximum
                    },
                    output(&mut entities, &mut clients)
                )
                .expect_err("Record budget"),
                expected
            );
            assert_eq!(reader.byte_position(), 0);
            assert_eq!((&entities, &clients), (&saved.0, &saved.1));
        }
        SnapshotServicePlan::prepare(
            &mut reader,
            input(context, 1, None),
            ServiceControlBudget { max_records: 4 },
            output(&mut entities, &mut clients),
        )
        .expect("Exact budget")
        .commit_transmitted()
        .expect("Commit");
    }
}
#[test]
fn missing_eof_is_reported_before_exposing_a_prediction_request_or_output() {
    for context in contexts() {
        let mut bytes = carrier(false);
        bytes.pop();
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Carrier");
        let (mut entities, mut clients) = dirty();
        let saved = (entities.clone(), clients.clone());
        assert_eq!(
            SnapshotServicePlan::prepare(
                &mut reader,
                input(context, 1, None),
                ServiceControlBudget::default(),
                output(&mut entities, &mut clients)
            )
            .expect_err("No EOF"),
            SnapshotServiceError::MissingEof
        );
        assert_eq!(reader.byte_position(), 0);
        assert_eq!((entities, clients), saved);
    }
}
#[test]
fn drop_and_prediction_failures_preserve_the_entire_carrier_then_explicit_fallback_commits() {
    for context in contexts() {
        let bytes = carrier(false);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Carrier");
        let (mut entities, mut clients) = dirty();
        let saved = (entities.clone(), clients.clone());
        let prepared = SnapshotServicePlan::prepare(
            &mut reader,
            input(context, 1, None),
            ServiceControlBudget::default(),
            output(&mut entities, &mut clients),
        )
        .expect("Prepared");
        drop(prepared);
        assert_eq!(reader.byte_position(), 0);
        assert_eq!((&entities, &clients), (&saved.0, &saved.1));
        let prepared = SnapshotServicePlan::prepare(
            &mut reader,
            input(context, 1, None),
            ServiceControlBudget::default(),
            output(&mut entities, &mut clients),
        )
        .expect("Retry");
        let wrong = PredictionSample::new(context.profile(), 1, sample(context.profile()).words());
        assert_eq!(
            prepared
                .commit_with_prediction(&wrong)
                .expect_err("Wrong sample"),
            SnapshotServiceError::Snapshot(SnapshotError::Player(PlayerStateError::PredictionTime))
        );
        assert_eq!(reader.byte_position(), 0);
        assert_eq!((&entities, &clients), (&saved.0, &saved.1));
        let prepared = SnapshotServicePlan::prepare(
            &mut reader,
            input(context, 1, None),
            ServiceControlBudget::default(),
            output(&mut entities, &mut clients),
        )
        .expect("Retry");
        assert_eq!(
            prepared.commit_transmitted().expect_err("Unresolved"),
            SnapshotServiceError::Snapshot(SnapshotError::Player(
                PlayerStateError::PredictionRequired
            ))
        );
        assert_eq!(reader.byte_position(), 0);
        assert_eq!((&entities, &clients), (&saved.0, &saved.1));
        SnapshotServicePlan::prepare(
            &mut reader,
            input(context, 1, None),
            ServiceControlBudget::default(),
            output(&mut entities, &mut clients),
        )
        .expect("Retry")
        .commit_native_prior_fallback()
        .expect("Explicit fallback");
    }
}
