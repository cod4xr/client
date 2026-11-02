// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Change, body, contexts, dirty, input, output, sample, values, word};
use crate::service_support::Fixture;
use cod4xr_client_core::{
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    service_controls::ServiceControlBudget,
    snapshots::services::{SnapshotPayloadPlan, SnapshotPrediction},
};

pub(super) fn carrier(transmitted: bool) -> Vec<u8> {
    let mut bytes = vec![6];
    bytes.extend(body(
        0,
        transmitted,
        &[Change::Set(1, 0, 13)],
        &[Change::Set(63, 0, 511)],
    ));
    bytes.push(7);
    bytes
}
#[test]
fn decoded_payload_binds_both_profiles_and_positive_ack_without_releasing_its_key() {
    for context in contexts() {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let mut outgoing = LegacyClientCommands::new(&mut raw).expect("Command owner");
        assert_eq!(outgoing.enqueue(b"SnapshotClientKey").expect("Key"), 1);
        outgoing.mark_transmitted(1).expect("Sent prefix");
        let (mut entities, mut clients) = dirty();
        let plan = {
            let mut fixture = Fixture::new(&carrier(true), 1, b"SnapshotClientKey");
            let payload = fixture.decode_profile(&outgoing, context.profile());
            SnapshotPayloadPlan::parse(
                &payload,
                input(context, 7, None),
                ServiceControlBudget::default(),
                output(&mut entities, &mut clients),
                SnapshotPrediction::Transmitted,
            )
            .expect("Actual decoded payload")
        };
        assert_eq!(plan.message_sequence(), 7);
        assert_eq!(plan.client_command_acknowledge(), 1);
        assert_eq!(
            plan.services().snapshot().header().profile(),
            context.profile()
        );
        assert_eq!(
            values(plan.services().snapshot()),
            (vec![(1, 13)], vec![(63, 511)])
        );
        assert_eq!(outgoing.acknowledged_sequence(), 0);
        assert_eq!(outgoing.pending_count(), 1);
        outgoing
            .commit_acknowledgement(plan.client_command_acknowledge())
            .expect("Explicit same-owner ACK");
        assert_eq!(outgoing.acknowledged_sequence(), 1);
        assert_eq!(values(plan.services().snapshot()).0, [(1, 13)]);
    }
}
#[test]
fn actual_decoded_carrier_supports_exact_sample_and_explicit_prior_fallback() {
    for context in contexts() {
        for fallback in [false, true] {
            let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
            let outgoing = LegacyClientCommands::new(&mut raw).expect("Command owner");
            let (mut entities, mut clients) = dirty();
            let mut fixture = Fixture::new(&carrier(false), 0, b"");
            let payload = fixture.decode_profile(&outgoing, context.profile());
            let sample = sample(context.profile());
            let decision = if fallback {
                SnapshotPrediction::NativePriorFallback
            } else {
                SnapshotPrediction::Sample(&sample)
            };
            let plan = SnapshotPayloadPlan::parse(
                &payload,
                input(context, 7, None),
                ServiceControlBudget::default(),
                output(&mut entities, &mut clients),
                decision,
            )
            .expect("Explicit selection");
            assert_eq!(
                plan.services()
                    .snapshot()
                    .player()
                    .scalars()
                    .word(word::motion::ORIGIN_X),
                u32::from(!fallback)
            );
            assert_eq!(outgoing.acknowledged_sequence(), 0);
        }
    }
}
#[test]
fn payload_plan_outlives_decoder_key_history_and_source_and_keeps_private_debug() {
    for context in contexts() {
        let (mut entities, mut clients) = dirty();
        let plan = {
            let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
            let mut outgoing = LegacyClientCommands::new(&mut raw).expect("Command owner");
            outgoing.enqueue(b"PrivateSnapshotKey").expect("Key");
            outgoing.mark_transmitted(1).expect("Sent prefix");
            let mut fixture = Fixture::new(&carrier(true), 1, b"PrivateSnapshotKey");
            let payload = fixture.decode_profile(&outgoing, context.profile());
            SnapshotPayloadPlan::parse(
                &payload,
                input(context, 7, None),
                ServiceControlBudget::default(),
                output(&mut entities, &mut clients),
                SnapshotPrediction::Transmitted,
            )
            .expect("Owned cooked plan")
        };
        let debug = format!("{plan:?}");
        assert!(!debug.contains("PrivateSnapshotKey"));
        assert!(!debug.contains("1985229328"));
        assert!(!debug.contains("511"));
        assert_eq!(
            plan.services().snapshot().as_source().message().sequence(),
            7
        );
    }
}
