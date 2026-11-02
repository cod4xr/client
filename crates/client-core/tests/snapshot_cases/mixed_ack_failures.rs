// SPDX-License-Identifier: AGPL-3.0-or-later
use super::mixed_support::{Finals, LIMITS, carrier, command, identity, selected, snapshot};
use super::{
    EntityDecodeContext, PlayerStateError, PredictionSample, SnapshotError, contexts, encoded,
    sample,
};
use crate::service_support::{Fixture, seed_identities};
use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4xr_client_core::{
    BaselineProtocol,
    client_identities::{
        ClientIdentityBudget, ClientIdentityError, ClientIdentitySlot, ClientIdentityState,
    },
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    server_commands::{SERVER_COMMAND_SLOT_BYTES, ServerCommandHistory, ServerCommandSlot},
    server_payload::{
        MixedSnapshotAcknowledgementError, MixedSnapshotRequest, ServerPayloadDecoder,
        SnapshotDecodeStorage,
    },
    service_controls::{ClientDataUpdateSlot, ServiceControlError},
    snapshots::{mixed_services::MixedSnapshotError, services::SnapshotPrediction},
};

#[test]
fn late_wire_and_prediction_errors_preserve_all_four_outputs_and_original_ack_key() {
    for context in contexts() {
        for bad_prediction in [false, true] {
            let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
            let mut history = LegacyClientCommands::new(&mut raw).expect("Original owner");
            history.enqueue(b"PrivateMixedKey").expect("Key");
            history.mark_transmitted(1).expect("Sent");
            let bytes = if bad_prediction {
                carrier(context.profile(), 0, false)
            } else {
                encoded(|writer| {
                    command(writer, 4, b"PrivateBefore");
                    snapshot(writer, 0, true);
                    command(writer, 0, b"InvalidTail");
                    writer.write_u8(7).expect("EOF");
                })
            };
            let fixture = Fixture::new(&bytes, 1, b"PrivateMixedKey");
            let payload_codec = ServerPayloadDecoder::new(context.profile(), 0x1234_5678);
            let mut work = vec![0; MAX_MESSAGE_BYTES];
            let mut decoded = vec![0; MAX_MESSAGE_BYTES];
            let wrong =
                PredictionSample::new(context.profile(), 1, sample(context.profile()).words());
            let mut finals = Finals::new();
            let saved = finals.clone();
            let mut text = [0; 128];
            let mut updates = vec![ClientDataUpdateSlot::default(); 4];
            let decision = if bad_prediction {
                SnapshotPrediction::Sample(&wrong)
            } else {
                SnapshotPrediction::Transmitted
            };
            let error = payload_codec
                .prepare_mixed_snapshot_ack(
                    &mut history,
                    7,
                    fixture.packet(),
                    SnapshotDecodeStorage {
                        work: &mut work,
                        decoded: &mut decoded,
                    },
                    MixedSnapshotRequest {
                        input: selected(context, 7),
                        services: LIMITS,
                        prediction: decision,
                    },
                    finals.storage(&mut text, &mut updates),
                )
                .expect_err("Late carrier failure");
            let expected = if bad_prediction {
                MixedSnapshotError::Snapshot(SnapshotError::Player(
                    PlayerStateError::PredictionTime,
                ))
            } else {
                MixedSnapshotError::Controls(ServiceControlError::Sequence)
            };
            assert_eq!(error, MixedSnapshotAcknowledgementError::Payload(expected));
            assert_eq!(finals, saved);
            assert_eq!(history.acknowledged_sequence(), 0);
            assert_eq!(history.pending_count(), 1);
        }
    }
}
#[test]
fn identity_owner_rejection_after_successful_wire_decode_cancels_other_preparations_and_ack() {
    let context = EntityDecodeContext::Cod4x21;
    let bytes = encoded(|writer| {
        command(writer, 4, b"PrivateBefore");
        identity(writer, 1, b"Before");
        snapshot(writer, 0, true);
        identity(writer, 2, b"After");
        command(writer, 9, b"PrivateAfter");
        writer.write_u8(7).expect("EOF");
    });
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut raw).expect("Original owner");
    history.enqueue(b"PrivateMixedKey").expect("Key");
    history.mark_transmitted(1).expect("Sent");
    let fixture = Fixture::new(&bytes, 1, b"PrivateMixedKey");
    let payload_codec = ServerPayloadDecoder::new(context.profile(), 0x1234_5678);
    let mut work = vec![0; MAX_MESSAGE_BYTES];
    let mut decoded = vec![0; MAX_MESSAGE_BYTES];
    let mut finals = Finals::new();
    let mut text = [0; 128];
    let mut updates = vec![ClientDataUpdateSlot::default(); 4];
    let pending = payload_codec
        .prepare_mixed_snapshot_ack(
            &mut history,
            7,
            fixture.packet(),
            SnapshotDecodeStorage {
                work: &mut work,
                decoded: &mut decoded,
            },
            MixedSnapshotRequest {
                input: selected(context, 7),
                services: LIMITS,
                prediction: SnapshotPrediction::Transmitted,
            },
            finals.storage(&mut text, &mut updates),
        )
        .expect("Syntactically valid cooked carrier");
    let mut server_bytes = vec![0; 2 * SERVER_COMMAND_SLOT_BYTES];
    let mut server_slots = [ServerCommandSlot::default(); 2];
    let mut server =
        ServerCommandHistory::new(context.profile(), &mut server_bytes, &mut server_slots)
            .expect("Command owner");
    let mut cells = vec![ClientIdentitySlot::default(); 2];
    let mut identities =
        ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut cells).expect("Identity owner");
    seed_identities(&mut identities, 0);
    let mut identity_work = vec![ClientIdentitySlot::default(); 2];
    let commands = server
        .prepare(pending.plan().controls())
        .expect("Command resource preflight");
    assert_eq!(
        identities
            .prepare_service_plan(
                pending.plan().controls(),
                ClientIdentityBudget {
                    max_clients: 2,
                    max_records: 1
                },
                &mut identity_work
            )
            .expect_err("Identity resource budget"),
        ClientIdentityError::RecordBudget
    );
    drop(commands);
    drop(pending);
    assert_eq!(server.newest_sequence(), 0);
    assert_eq!(identities.accepted_sequence(), Some(0));
    assert!(identities.is_empty());
    assert_eq!(history.acknowledged_sequence(), 0);
    assert_eq!(history.pending_count(), 1);
    assert_eq!(
        history.command(1).expect("Original key").as_bytes(),
        b"PrivateMixedKey"
    );
}
