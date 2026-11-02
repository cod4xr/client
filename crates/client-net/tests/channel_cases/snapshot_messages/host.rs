// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_client_core::{
    client_identities::{
        ClientIdentityBudget, ClientIdentityError, ClientIdentitySlot, ClientIdentityState,
    },
    client_states::ClientStateWord,
    entity_deltas::EntityWord,
    server_commands::ServerCommandHistory,
    server_payload::PreparedMixedSnapshotAcknowledgement,
    snapshots::mixed_services::MixedSnapshotPayloadPlan,
};

pub(super) fn commit_after_resource_retry<'output>(
    pending: PreparedMixedSnapshotAcknowledgement<'_, '_, 'output>,
    commands: &mut ServerCommandHistory<'_>,
    identities: &mut ClientIdentityState<'_>,
) -> MixedSnapshotPayloadPlan<'output> {
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
    assert_eq!(commands.newest_sequence(), 0);
    assert_eq!(identities.accepted_sequence(), Some(9));
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
    let _ = prepared_commands.commit();
    let _ = prepared_identities.commit();
    pending.commit()
}

pub(super) fn assert_snapshot(plan: &MixedSnapshotPayloadPlan<'_>) {
    assert_eq!(plan.snapshot().message().server_command_sequence(), 3);
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
