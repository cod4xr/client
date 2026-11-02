// SPDX-License-Identifier: AGPL-3.0-or-later
use super::receive::{Finals, receive};
use cod4xr_client_core::{
    BaselineProtocol,
    client_identities::{
        ClientIdentityBudget, ClientIdentityError, ClientIdentitySlot, ClientIdentityState,
    },
    client_states::ClientStateWord,
    commands::LegacyClientCommands,
    entity_deltas::EntityWord,
    player_scalars::word,
    player_state::{
        PredictionProvenance, PredictionSample, PredictionWords, history::PredictionHistory,
    },
    server_commands::ServerCommandHistory,
    server_payload::{MixedSnapshotAcknowledgementError, PreparedMixedSnapshotAcknowledgement},
    service_controls::ServiceControlError,
    snapshots::{
        history::SnapshotHistory,
        mixed_services::{MixedSnapshotError, MixedSnapshotPayloadPlan},
    },
};
use cod4xr_client_net::ClientChannel;

pub(super) fn commit_after_resource_retry<'output>(
    pending: PreparedMixedSnapshotAcknowledgement<'_, '_, 'output>,
    commands: &mut ServerCommandHistory<'_>,
    identities: &mut ClientIdentityState<'_>,
    snapshots: &mut SnapshotHistory<'_, '_>,
) -> MixedSnapshotPayloadPlan<'output> {
    let original_command = commands.newest_sequence();
    let original_identity = identities.accepted_sequence();
    let original_snapshot = snapshots.newest_sequence();
    let mut identity_work = vec![ClientIdentitySlot::default(); 2];
    let prepared_commands = commands
        .prepare(pending.plan().controls())
        .expect("Command preflight");
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
            .expect_err("Host identity resource budget"),
        ClientIdentityError::RecordBudget
    );
    drop(prepared_commands);
    assert_eq!(commands.newest_sequence(), original_command);
    assert_eq!(identities.accepted_sequence(), original_identity);
    assert_eq!(snapshots.newest_sequence(), original_snapshot);
    let prepared_commands = commands
        .prepare(pending.plan().controls())
        .expect("Retry command resources");
    let prepared_identities = identities
        .prepare_service_plan(
            pending.plan().controls(),
            ClientIdentityBudget {
                max_clients: 2,
                max_records: 2,
            },
            &mut identity_work,
        )
        .expect("Retry identity resources");
    let prepared_snapshot = snapshots
        .prepare(pending.plan().snapshot())
        .expect("Snapshot history resources");
    let _ = prepared_commands.commit();
    let _ = prepared_identities.commit();
    let _ = prepared_snapshot.commit();
    pending.commit()
}

pub(super) fn assert_snapshot(
    plan: &MixedSnapshotPayloadPlan<'_>,
    command_sequence: u32,
    origin: u32,
) {
    assert_eq!(
        plan.snapshot().player().prediction(),
        PredictionProvenance::CommandTimeMatched
    );
    assert_eq!(
        plan.snapshot()
            .player()
            .scalars()
            .word(word::motion::ORIGIN_X),
        origin
    );
    assert_eq!(
        plan.snapshot().message().server_command_sequence(),
        command_sequence
    );
    assert_eq!(
        plan.snapshot()
            .entities()
            .entities()
            .map(|state| (state.number(), state.word(EntityWord::Kind)))
            .collect::<Vec<_>>(),
        [(1, 13)]
    );
    assert_eq!(
        plan.snapshot()
            .clients()
            .clients()
            .map(|state| (state.number(), state.word(ClientStateWord::Model)))
            .collect::<Vec<_>>(),
        [(63, 511)]
    );
}

pub(super) fn prediction_sample(origin: f32) -> PredictionSample {
    PredictionSample::new(
        BaselineProtocol::Cod4x21,
        0,
        PredictionWords::new([origin.to_bits(), 0, 0], [0; 3], [0; 3], 0, 0),
    )
}

pub(super) fn reject_malformed(
    channel: &mut ClientChannel<'_>,
    outgoing: &mut LegacyClientCommands<'_>,
    commands: &ServerCommandHistory<'_>,
    identities: &ClientIdentityState<'_>,
    snapshots: &SnapshotHistory<'_, '_>,
    prediction: &PredictionHistory<'_>,
    finals: &mut Finals,
) {
    let saved = finals.clone();
    assert_eq!(
        receive(
            channel,
            outgoing,
            commands.newest_sequence(),
            snapshots,
            prediction,
            finals
        )
        .expect_err("Late malformed service before missing prediction"),
        MixedSnapshotAcknowledgementError::Payload(MixedSnapshotError::Controls(
            ServiceControlError::UnsupportedService(0xfe)
        ))
    );
    assert_eq!(*finals, saved);
    assert_eq!(
        (outgoing.acknowledged_sequence(), outgoing.pending_count()),
        (0, 1)
    );
    assert_eq!(commands.newest_sequence(), 0);
    assert_eq!(identities.accepted_sequence(), Some(9));
    assert!(identities.is_empty());
    assert_eq!(channel.split().receiver.incoming_sequence(), 7);
    assert!(prediction.is_empty());
}

pub(super) fn assert_owners(
    outgoing: &LegacyClientCommands<'_>,
    commands: &ServerCommandHistory<'_>,
    identities: &ClientIdentityState<'_>,
    sequence: u32,
    acknowledgement: u32,
) {
    assert_eq!(
        (outgoing.acknowledged_sequence(), outgoing.pending_count()),
        (acknowledgement, 0)
    );
    assert_eq!(
        (commands.newest_sequence(), commands.last_message_sequence()),
        (4, sequence)
    );
    assert_eq!(identities.accepted_sequence(), Some(11));
    assert_eq!(
        identities.client(3).expect("Retained identity").name(),
        b"After"
    );
}
