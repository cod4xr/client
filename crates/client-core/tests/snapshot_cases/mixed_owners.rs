// SPDX-License-Identifier: AGPL-3.0-or-later
use super::mixed_support::{Finals, LIMITS, carrier, selected};
use super::{EntityDecodeContext, SnapshotMessage};
use crate::service_support::{Fixture, seed_identities};
use cod4xr_client_core::{
    BaselineProtocol,
    client_identities::{ClientIdentityBudget, ClientIdentitySlot, ClientIdentityState},
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    server_commands::{SERVER_COMMAND_SLOT_BYTES, ServerCommandHistory, ServerCommandSlot},
    service_controls::ClientDataUpdateSlot,
    snapshots::{mixed_services::MixedSnapshotPayloadPlan, services::SnapshotPrediction},
};

#[test]
fn cooked_mixed_plan_preflights_both_existing_owners_before_any_history_commit() {
    let context = EntityDecodeContext::Cod4x21;
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let outgoing = LegacyClientCommands::new(&mut raw).expect("Outgoing owner");
    let mut fixture = Fixture::new(&carrier(context.profile(), 0, true), 0, b"");
    let payload = fixture.decode(&outgoing);
    let mut server_bytes = vec![0; 2 * SERVER_COMMAND_SLOT_BYTES];
    let mut server_slots = [ServerCommandSlot::default(); 2];
    let mut server =
        ServerCommandHistory::new(context.profile(), &mut server_bytes, &mut server_slots)
            .expect("Command owner");
    let mut cells = vec![ClientIdentitySlot::default(); 2];
    let mut identities =
        ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut cells).expect("Identity owner");
    seed_identities(&mut identities, 0);
    let mut request = selected(context, 7);
    request.message = SnapshotMessage::new(7, 7, server.newest_sequence())
        .expect("Actual initial high-water mark");
    let mut finals = Finals::new();
    let mut text = [0; 128];
    let mut updates = vec![ClientDataUpdateSlot::default(); 4];
    let plan = MixedSnapshotPayloadPlan::parse(
        &payload,
        request,
        LIMITS,
        finals.storage(&mut text, &mut updates),
        SnapshotPrediction::Transmitted,
    )
    .expect("Cooked mixed plan");
    let mut identity_work = vec![ClientIdentitySlot::default(); 2];
    let budget = ClientIdentityBudget {
        max_clients: 2,
        max_records: 2,
    };
    let commands = server.prepare(plan.controls()).expect("Command preflight");
    let patches = identities
        .prepare_service_plan(plan.controls(), budget, &mut identity_work)
        .expect("Identity preflight");
    assert_eq!(
        commands
            .new_commands()
            .map(|command| command.sequence())
            .collect::<Vec<_>>(),
        [4, 9]
    );
    drop(patches);
    drop(commands);
    assert_eq!(server.newest_sequence(), 0);
    assert_eq!(identities.accepted_sequence(), Some(0));
    assert!(identities.is_empty());
    let commands = server.prepare(plan.controls()).expect("Retry commands");
    let patches = identities
        .prepare_service_plan(plan.controls(), budget, &mut identity_work)
        .expect("Retry identities");
    let command_update = commands.commit();
    let _ = patches.commit();
    assert_eq!(command_update.new_commands, 2);
    assert_eq!(command_update.ignored_commands, 1);
    assert_eq!(server.newest_sequence(), 9);
    assert_eq!(identities.accepted_sequence(), Some(2));
    assert_eq!(
        identities.client(2).expect("Final identity").name(),
        b"After"
    );
    assert_eq!(plan.snapshot().message().server_command_sequence(), 4);
    assert_eq!(outgoing.acknowledged_sequence(), 0);
}
