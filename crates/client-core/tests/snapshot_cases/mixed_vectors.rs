// SPDX-License-Identifier: AGPL-3.0-or-later
use super::mixed_support::{Finals, LIMITS, carrier, selected};
use super::{SnapshotMessage, contexts, sample, values};
use crate::service_support::Fixture;
use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    service_controls::ClientDataUpdateSlot,
    snapshots::{mixed_services::MixedSnapshotPayloadPlan, services::SnapshotPrediction},
};

#[test]
fn commands_and_identities_on_both_sides_preserve_native_snapshot_command_prefix() {
    for context in contexts() {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let outgoing = LegacyClientCommands::new(&mut raw).expect("Original owner");
        let mut fixture = Fixture::new(&carrier(context.profile(), 0, true), 0, b"");
        let payload = fixture.decode_profile(&outgoing, context.profile());
        let mut finals = Finals::new();
        let saved = finals.clone();
        let mut text_scratch = [0; 128];
        let mut identity_scratch = vec![ClientDataUpdateSlot::default(); 4];
        let plan = MixedSnapshotPayloadPlan::parse(
            &payload,
            selected(context, 7),
            LIMITS,
            finals.storage(&mut text_scratch, &mut identity_scratch),
            SnapshotPrediction::Transmitted,
        )
        .expect("Whole mixed carrier");
        let commands: Vec<_> = plan
            .controls()
            .commands()
            .map(|command| (command.sequence(), command.as_bytes().to_vec()))
            .collect();
        assert_eq!(
            commands,
            [
                (4, b"alpha.".to_vec()),
                (2, b"older".to_vec()),
                (9, b"omega".to_vec())
            ]
        );
        assert_eq!(plan.snapshot().message().server_command_sequence(), 4);
        assert_eq!(plan.controls().message_sequence(), 7);
        assert_eq!(values(plan.snapshot()), (vec![(1, 13)], vec![(63, 511)]));
        assert_eq!(
            plan.controls().record_count(),
            if context.profile() == BaselineProtocol::Cod4x21 {
                9
            } else {
                7
            }
        );
        assert_eq!(
            plan.controls().client_update_count(),
            if context.profile() == BaselineProtocol::Cod4x21 {
                2
            } else {
                0
            }
        );
        assert_eq!(outgoing.acknowledged_sequence(), 0);
        let updates: Vec<_> = plan
            .controls()
            .client_updates()
            .map(|update| (update.sequence(), update.data().name().to_vec()))
            .collect();
        assert_eq!(
            updates,
            if context.profile() == BaselineProtocol::Cod4x21 {
                vec![(1, b"Before.".to_vec()), (2, b"After".to_vec())]
            } else {
                vec![]
            }
        );
        let debug = format!("{plan:?}");
        assert!(!debug.contains("alpha"));
        assert!(!debug.contains("Before"));
        assert!(!debug.contains("After"));
        let command_bytes = plan.controls().command_bytes();
        let identities = plan.controls().client_update_count();
        assert_eq!(&finals.text[command_bytes..], &saved.text[command_bytes..]);
        assert_eq!(
            &finals.identities[identities..],
            &saved.identities[identities..]
        );
        assert_eq!(&finals.entities[1..], &saved.entities[1..]);
        assert_eq!(&finals.clients[1..], &saved.clients[1..]);
    }
}
#[test]
fn mixed_result_outlives_payload_history_and_scratch_and_supplies_a_coherent_delta_source() {
    for context in contexts() {
        let mut old_finals = Finals::new();
        let old = {
            let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
            let outgoing = LegacyClientCommands::new(&mut raw).expect("Owner");
            let mut fixture =
                Fixture::at_sequence(&carrier(context.profile(), 0, false), 0, b"", 10);
            let payload = fixture.decode_profile(&outgoing, context.profile());
            let mut text = [0; 128];
            let mut updates = vec![ClientDataUpdateSlot::default(); 4];
            let sample = sample(context.profile());
            MixedSnapshotPayloadPlan::parse(
                &payload,
                selected(context, 10),
                LIMITS,
                old_finals.storage(&mut text, &mut updates),
                SnapshotPrediction::Sample(&sample),
            )
            .expect("Owned mixed result")
        };
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let outgoing = LegacyClientCommands::new(&mut raw).expect("Owner");
        let mut fixture = Fixture::at_sequence(&carrier(context.profile(), 1, true), 0, b"", 11);
        let payload = fixture.decode_profile(&outgoing, context.profile());
        let mut selected = selected(context, 11);
        selected.previous = Some(old.snapshot().as_source());
        selected.message = SnapshotMessage::new(7, 11, 9).expect("Actual host high-water mark");
        let mut finals = Finals::new();
        let mut text = [0; 128];
        let mut updates = vec![ClientDataUpdateSlot::default(); 4];
        let next = MixedSnapshotPayloadPlan::parse(
            &payload,
            selected,
            LIMITS,
            finals.storage(&mut text, &mut updates),
            SnapshotPrediction::Transmitted,
        )
        .expect("Coherent delta source");
        assert_eq!(values(next.snapshot()), values(old.snapshot()));
        assert_eq!(next.snapshot().message().server_command_sequence(), 9);
        assert_eq!(
            next.snapshot().player().scalars(),
            old.snapshot().player().scalars()
        );
    }
}
