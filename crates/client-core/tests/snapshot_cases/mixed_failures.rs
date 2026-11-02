// SPDX-License-Identifier: AGPL-3.0-or-later
use super::mixed_support::{Finals, LIMITS, command, identity, selected, snapshot};
use super::{PlayerStateError, PredictionSample, SnapshotError, contexts, encoded, sample};
use crate::service_support::Fixture;
use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    service_controls::{ClientDataUpdateSlot, ServiceControlError},
    snapshots::{
        mixed_services::{MixedSnapshotError, MixedSnapshotPayloadPlan},
        services::SnapshotPrediction,
    },
};

#[test]
fn late_control_failure_preserves_command_identity_entity_and_client_outputs() {
    for context in contexts() {
        let bytes = encoded(|writer| {
            command(writer, 4, b"PrivateBefore");
            if context.profile() == BaselineProtocol::Cod4x21 {
                identity(writer, 1, b"PrivateIdentity");
            }
            snapshot(writer, 0, true);
            command(writer, 0, b"InvalidSequence");
            writer.write_u8(7).expect("EOF");
        });
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let outgoing = LegacyClientCommands::new(&mut raw).expect("Owner");
        let mut fixture = Fixture::new(&bytes, 0, b"");
        let payload = fixture.decode_profile(&outgoing, context.profile());
        let mut finals = Finals::new();
        let saved = finals.clone();
        let mut text = [0; 128];
        let mut updates = vec![ClientDataUpdateSlot::default(); 4];
        assert_eq!(
            MixedSnapshotPayloadPlan::parse(
                &payload,
                selected(context, 7),
                LIMITS,
                finals.storage(&mut text, &mut updates),
                SnapshotPrediction::Transmitted
            )
            .expect_err("Late control failure"),
            MixedSnapshotError::Controls(ServiceControlError::Sequence)
        );
        assert_eq!(finals, saved);
        assert_eq!(outgoing.acknowledged_sequence(), 0);
    }
}
#[test]
fn prediction_failure_after_valid_tail_controls_preserves_all_four_final_buffers() {
    for context in contexts() {
        let bytes = super::mixed_support::carrier(context.profile(), 0, false);
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let outgoing = LegacyClientCommands::new(&mut raw).expect("Owner");
        let mut fixture = Fixture::new(&bytes, 0, b"");
        let payload = fixture.decode_profile(&outgoing, context.profile());
        let mut finals = Finals::new();
        let saved = finals.clone();
        let mut text = [0; 128];
        let mut updates = vec![ClientDataUpdateSlot::default(); 4];
        let wrong = PredictionSample::new(context.profile(), 1, sample(context.profile()).words());
        assert_eq!(
            MixedSnapshotPayloadPlan::parse(
                &payload,
                selected(context, 7),
                LIMITS,
                finals.storage(&mut text, &mut updates),
                SnapshotPrediction::Sample(&wrong)
            )
            .expect_err("Late selection failure"),
            MixedSnapshotError::Snapshot(SnapshotError::Player(PlayerStateError::PredictionTime))
        );
        assert_eq!(finals, saved);
        assert_eq!(outgoing.acknowledged_sequence(), 0);
    }
}
