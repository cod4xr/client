// SPDX-License-Identifier: AGPL-3.0-or-later
use super::payload_vectors::carrier;
use super::{
    PlayerStateError, PredictionSample, SnapshotError, contexts, dirty, input, output, sample,
};
use crate::service_support::Fixture;
use cod4xr_client_core::{
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    service_controls::ServiceControlBudget,
    snapshots::services::{
        SnapshotPayloadError, SnapshotPayloadPlan, SnapshotPrediction, SnapshotServiceError,
    },
};

#[test]
fn actual_profile_and_sequence_mismatches_precede_service_work_and_final_output_changes() {
    let contexts = contexts();
    for (index, &context) in contexts.iter().enumerate() {
        for wrong_profile in [false, true] {
            let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
            let outgoing = LegacyClientCommands::new(&mut raw).expect("Command owner");
            let mut fixture = Fixture::new(&carrier(true), 0, b"");
            let payload = fixture.decode_profile(&outgoing, context.profile());
            let selected = if wrong_profile {
                input(contexts[1 - index], 7, None)
            } else {
                input(context, 8, None)
            };
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            let error = SnapshotPayloadPlan::parse(
                &payload,
                selected,
                ServiceControlBudget { max_records: 0 },
                output(&mut entities, &mut clients),
                SnapshotPrediction::Transmitted,
            )
            .expect_err("Actual payload binding");
            assert_eq!(
                error,
                if wrong_profile {
                    SnapshotPayloadError::Profile
                } else {
                    SnapshotPayloadError::MessageSequence
                }
            );
            assert_eq!((entities, clients), saved);
            assert_eq!(outgoing.acknowledged_sequence(), 0);
        }
    }
}
#[test]
fn unsupported_decoded_tail_preserves_outputs_and_positive_ack_key_then_allows_retry() {
    for context in contexts() {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let mut outgoing = LegacyClientCommands::new(&mut raw).expect("Command owner");
        outgoing.enqueue(b"SnapshotClientKey").expect("Key");
        outgoing.mark_transmitted(1).expect("Sent");
        let mut bytes = carrier(true);
        let last = bytes.len() - 1;
        bytes[last] = 11;
        let (mut entities, mut clients) = dirty();
        let saved = (entities.clone(), clients.clone());
        {
            let mut fixture = Fixture::new(&bytes, 1, b"SnapshotClientKey");
            let payload = fixture.decode_profile(&outgoing, context.profile());
            assert_eq!(
                SnapshotPayloadPlan::parse(
                    &payload,
                    input(context, 7, None),
                    ServiceControlBudget::default(),
                    output(&mut entities, &mut clients),
                    SnapshotPrediction::Transmitted
                )
                .expect_err("Unsupported tail"),
                SnapshotPayloadError::Services(SnapshotServiceError::UnsupportedService(11))
            );
        }
        assert_eq!((&entities, &clients), (&saved.0, &saved.1));
        assert_eq!(outgoing.acknowledged_sequence(), 0);
        assert_eq!(outgoing.pending_count(), 1);
        let mut fixture = Fixture::new(&carrier(true), 1, b"SnapshotClientKey");
        let payload = fixture.decode_profile(&outgoing, context.profile());
        SnapshotPayloadPlan::parse(
            &payload,
            input(context, 7, None),
            ServiceControlBudget::default(),
            output(&mut entities, &mut clients),
            SnapshotPrediction::Transmitted,
        )
        .expect("Valid retry");
        assert_eq!(outgoing.acknowledged_sequence(), 0);
    }
}
#[test]
fn invalid_decoded_prediction_choice_preserves_both_outputs_and_retained_command_owner() {
    for context in contexts() {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let outgoing = LegacyClientCommands::new(&mut raw).expect("Command owner");
        let mut fixture = Fixture::new(&carrier(false), 0, b"");
        let payload = fixture.decode_profile(&outgoing, context.profile());
        let wrong = PredictionSample::new(context.profile(), 1, sample(context.profile()).words());
        let (mut entities, mut clients) = dirty();
        let saved = (entities.clone(), clients.clone());
        assert_eq!(
            SnapshotPayloadPlan::parse(
                &payload,
                input(context, 7, None),
                ServiceControlBudget::default(),
                output(&mut entities, &mut clients),
                SnapshotPrediction::Sample(&wrong)
            )
            .expect_err("Wrong sample"),
            SnapshotPayloadError::Services(SnapshotServiceError::Snapshot(SnapshotError::Player(
                PlayerStateError::PredictionTime
            )))
        );
        assert_eq!((entities, clients), saved);
        assert_eq!(outgoing.acknowledged_sequence(), 0);
    }
}
