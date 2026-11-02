// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, PredictionHistory, PredictionHistoryError as Error, PredictionSlot, SnapshotError,
    contexts, dirty, input, output, position, retain,
};
use crate::{service_support::Fixture, snapshots::payload_vectors::carrier};
use cod4xr_client_core::{
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    service_controls::ServiceControlBudget,
    snapshots::services::{
        SnapshotPayloadPlan, SnapshotPrediction, SnapshotServiceError, SnapshotServicePlan,
    },
};

#[test]
fn carrier_eof_validation_precedes_retained_lookup_and_transmitted_mode_still_checks_world() {
    for context in contexts() {
        for transmitted in [false, true] {
            let mut slots = [PredictionSlot::default()];
            let history =
                PredictionHistory::new(8, context.profile(), &mut slots).expect("Wrong world");
            let mut bytes = carrier(transmitted);
            *bytes.last_mut().expect("EOF") = 0xfe;
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            let before = position(&reader);
            assert_eq!(
                SnapshotServicePlan::prepare(
                    &mut reader,
                    input(context, 7, None),
                    ServiceControlBudget::default(),
                    output(&mut entities, &mut clients)
                )
                .expect_err("Late EOF before lookup"),
                SnapshotServiceError::UnsupportedService(0xfe)
            );
            assert_eq!(position(&reader), before);
            assert_eq!((&entities, &clients), (&saved.0, &saved.1));
            let bytes = carrier(transmitted);
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Retry body");
            let before = position(&reader);
            let error = SnapshotServicePlan::prepare(
                &mut reader,
                input(context, 7, None),
                ServiceControlBudget::default(),
                output(&mut entities, &mut clients),
            )
            .expect("Complete carrier")
            .commit_from_history(history.source())
            .expect_err("Explicit world binding in either mode");
            assert_eq!(
                error,
                SnapshotServiceError::Snapshot(SnapshotError::PredictionHistory(Error::Epoch))
            );
            assert_eq!(position(&reader), before);
            assert_eq!((entities, clients), saved);
        }
    }
}

#[test]
fn actual_payload_policy_resolves_unsent_mode_and_accepts_transmitted_mode_with_empty_bound_history()
 {
    for context in contexts() {
        for transmitted in [false, true] {
            let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
            let outgoing = LegacyClientCommands::new(&mut raw).expect("Outgoing owner");
            let mut slots = [PredictionSlot::default()];
            let mut history =
                PredictionHistory::new(7, context.profile(), &mut slots).expect("Owner");
            if !transmitted {
                retain(&mut history, 0, 999);
            }
            let (mut entities, mut clients) = dirty();
            let plan = {
                let mut fixture = Fixture::new(&carrier(transmitted), 0, b"");
                let payload = fixture.decode_profile(&outgoing, context.profile());
                SnapshotPayloadPlan::parse(
                    &payload,
                    input(context, 7, None),
                    ServiceControlBudget::default(),
                    output(&mut entities, &mut clients),
                    SnapshotPrediction::History(history.source()),
                )
                .expect("Actual decoded mode")
            };
            history
                .reset(8, context.profile())
                .expect("No retained source borrow");
            assert_eq!(plan.message_sequence(), 7);
            assert_eq!(
                plan.services()
                    .snapshot()
                    .player()
                    .scalars()
                    .word(super::word::motion::ORIGIN_X),
                if transmitted { 0 } else { 999 }
            );
            assert_eq!(outgoing.acknowledged_sequence(), 0);
        }
    }
}
