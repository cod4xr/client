// SPDX-License-Identifier: AGPL-3.0-or-later
use super::mixed_support::{Finals, LIMITS, carrier, selected};
use super::{EntityDecodeContext, SnapshotError};
use crate::service_support::Fixture;
use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    service_controls::{ClientDataUpdateSlot, ServiceControlError},
    snapshot_clients::ClientStreamError,
    snapshot_entities::EntityStreamError,
    snapshots::{
        mixed_services::{MixedSnapshotError, MixedSnapshotPayloadPlan},
        services::SnapshotPrediction,
    },
};

#[test]
fn independent_late_control_and_snapshot_limits_preserve_all_four_final_outputs() {
    let context = EntityDecodeContext::Cod4x21;
    for case in 0..8 {
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let outgoing = LegacyClientCommands::new(&mut raw).expect("Owner");
        let mut fixture = Fixture::new(&carrier(BaselineProtocol::Cod4x21, 0, true), 0, b"");
        let payload = fixture.decode(&outgoing);
        let mut limits = LIMITS;
        let mut finals = Finals::new();
        let mut text = [0; 128];
        let mut updates = vec![ClientDataUpdateSlot::default(); 4];
        let mut text_capacity = text.len();
        let expected = match case {
            0 => {
                limits.max_records = 8;
                Some(MixedSnapshotError::Controls(
                    ServiceControlError::RecordBudget,
                ))
            }
            1 => {
                limits.max_client_updates = 1;
                Some(MixedSnapshotError::Controls(
                    ServiceControlError::ClientDataBudget,
                ))
            }
            2 => {
                updates.truncate(1);
                Some(MixedSnapshotError::Controls(
                    ServiceControlError::ClientDataScratch,
                ))
            }
            3 => {
                finals.identities.truncate(1);
                Some(MixedSnapshotError::Controls(
                    ServiceControlError::ClientDataCapacity,
                ))
            }
            4 => {
                finals.text.truncate(14);
                Some(MixedSnapshotError::Controls(
                    ServiceControlError::OutputCapacity,
                ))
            }
            5 => {
                text_capacity = 5;
                None
            }
            6 => {
                finals.entities.clear();
                Some(MixedSnapshotError::Snapshot(SnapshotError::Entities(
                    EntityStreamError::OutputCapacity,
                )))
            }
            _ => {
                finals.clients.clear();
                Some(MixedSnapshotError::Snapshot(SnapshotError::Clients(
                    ClientStreamError::OutputCapacity,
                )))
            }
        };
        let saved = finals.clone();
        let error = MixedSnapshotPayloadPlan::parse(
            &payload,
            selected(context, 7),
            limits,
            finals.storage(&mut text[..text_capacity], &mut updates),
            SnapshotPrediction::Transmitted,
        )
        .expect_err("Independent capacity/work limit");
        if let Some(expected) = expected {
            assert_eq!(error, expected);
        } else {
            assert!(matches!(error, MixedSnapshotError::Controls(_)));
        }
        assert_eq!(finals, saved);
        assert_eq!(outgoing.acknowledged_sequence(), 0);
    }
}
#[test]
fn exact_total_record_budget_counts_the_snapshot_and_padding_stays_uninterpreted() {
    let context = EntityDecodeContext::Cod4x21;
    let mut bytes = carrier(context.profile(), 0, true);
    let logical = bytes.len();
    bytes.extend([4, 11, 6]);
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let outgoing = LegacyClientCommands::new(&mut raw).expect("Owner");
    let mut fixture = Fixture::new(&bytes, 0, b"");
    let payload = fixture.decode(&outgoing);
    let mut finals = Finals::new();
    let mut text = [0; 128];
    let mut updates = vec![ClientDataUpdateSlot::default(); 4];
    let mut limits = LIMITS;
    limits.max_records = 9;
    let plan = MixedSnapshotPayloadPlan::parse(
        &payload,
        selected(context, 7),
        limits,
        finals.storage(&mut text, &mut updates),
        SnapshotPrediction::Transmitted,
    )
    .expect("Exact full-carrier budget");
    assert_eq!(plan.controls().record_count(), 9);
    assert_eq!(plan.controls().logical_bytes(), logical);
    assert!(plan.snapshot().consumed_bytes() < logical);
}
