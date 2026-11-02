// SPDX-License-Identifier: AGPL-3.0-or-later
use super::mixed_support::{Finals, LIMITS, carrier, command, identity, selected, snapshot};
use super::{BaselineProtocol, EntityDecodeContext, contexts, encoded};
use crate::service_support::Fixture;
use cod4xr_client_core::{
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    service_controls::{ClientDataUpdateSlot, ServiceControlError},
    snapshots::{
        mixed_services::{MixedSnapshotError, MixedSnapshotPayloadPlan},
        services::{SnapshotPayloadError, SnapshotPrediction},
    },
};

#[test]
fn missing_snapshot_second_snapshot_and_unsupported_services_leave_all_outputs_untouched() {
    let context = EntityDecodeContext::Cod4x21;
    for case in 0..4 {
        let bytes = encoded(|writer| {
            command(writer, 4, b"PrivateCommand");
            identity(writer, 1, b"PrivateIdentity");
            if case != 0 {
                snapshot(writer, 0, true);
            }
            if case == 1 {
                writer.write_u8(6).expect("Second snapshot service");
            }
            if case == 2 {
                writer.write_u8(5).expect("Unsupported download service");
            }
            if case == 3 {
                writer.write_u8(4).expect("Truncated tail command");
            } else {
                writer.write_u8(7).expect("EOF");
            }
        });
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let outgoing = LegacyClientCommands::new(&mut raw).expect("Owner");
        let mut fixture = Fixture::new(&bytes, 0, b"");
        let payload = fixture.decode(&outgoing);
        let mut finals = Finals::new();
        let saved = finals.clone();
        let mut text = [0; 128];
        let mut updates = vec![ClientDataUpdateSlot::default(); 4];
        let error = MixedSnapshotPayloadPlan::parse(
            &payload,
            selected(context, 7),
            LIMITS,
            finals.storage(&mut text, &mut updates),
            SnapshotPrediction::Transmitted,
        )
        .expect_err("Incomplete/unsupported framing");
        match case {
            0 => assert_eq!(error, MixedSnapshotError::MissingSnapshot),
            1 => assert_eq!(
                error,
                MixedSnapshotError::Controls(ServiceControlError::UnsupportedService(6))
            ),
            2 => assert_eq!(
                error,
                MixedSnapshotError::Controls(ServiceControlError::UnsupportedService(5))
            ),
            _ => assert!(matches!(error, MixedSnapshotError::Controls(_))),
        }
        assert_eq!(finals, saved);
        assert_eq!(outgoing.acknowledged_sequence(), 0);
    }
}
#[test]
fn actual_payload_context_mismatch_rejects_before_any_control_work() {
    let contexts = contexts();
    for wrong_profile in [false, true] {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let outgoing = LegacyClientCommands::new(&mut raw).expect("Owner");
        let mut fixture = Fixture::new(&carrier(BaselineProtocol::Cod4x21, 0, true), 0, b"");
        let payload = fixture.decode(&outgoing);
        let chosen = if wrong_profile {
            selected(contexts[1], 7)
        } else {
            selected(contexts[0], 8)
        };
        let mut finals = Finals::new();
        let saved = finals.clone();
        let mut text = [];
        let mut updates = [];
        let mut limits = LIMITS;
        limits.max_records = 0;
        let expected = if wrong_profile {
            SnapshotPayloadError::Profile
        } else {
            SnapshotPayloadError::MessageSequence
        };
        assert_eq!(
            MixedSnapshotPayloadPlan::parse(
                &payload,
                chosen,
                limits,
                finals.storage(&mut text, &mut updates),
                SnapshotPrediction::Transmitted
            )
            .expect_err("Actual context binding"),
            MixedSnapshotError::Binding(expected)
        );
        assert_eq!(finals, saved);
    }
}
#[test]
fn legacy_identity_extension_is_rejected_without_inventing_a_schema() {
    let context = contexts()[1];
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let outgoing = LegacyClientCommands::new(&mut raw).expect("Owner");
    let mut fixture = Fixture::new(&carrier(BaselineProtocol::Cod4x21, 0, true), 0, b"");
    let payload = fixture.decode_profile(&outgoing, BaselineProtocol::Legacy6);
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
        .expect_err("Unreviewed legacy extension"),
        MixedSnapshotError::Controls(ServiceControlError::UnsupportedService(11))
    );
    assert_eq!(finals, saved);
}
