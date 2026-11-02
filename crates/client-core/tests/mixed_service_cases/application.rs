// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use cod4xr_client_core::server_commands::{
    SERVER_COMMAND_SLOT_BYTES, ServerCommandHistory, ServerCommandSlot,
};

#[test]
fn one_whole_plan_preflights_both_histories_then_retains_records_without_source_buffers() {
    let mut output = Output::new(64, 8);
    let plan = {
        let mut fixture = Fixture::new(&mixed(), 0, b"");
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let outgoing = LegacyClientCommands::new(&mut raw).expect("Outgoing");
        let payload = fixture.decode(&outgoing);
        LegacyServicePlan::parse_with_client_data(
            &payload,
            BUDGET,
            &mut [0; 64],
            output.storage(&mut vec![ClientDataUpdateSlot::default(); 8]),
        )
        .expect("Complete mixed message")
    };
    let mut server_bytes = vec![0; 2 * SERVER_COMMAND_SLOT_BYTES];
    let mut server_slots = [ServerCommandSlot::default(); 2];
    let mut server = ServerCommandHistory::new(
        BaselineProtocol::Cod4x21,
        &mut server_bytes,
        &mut server_slots,
    )
    .expect("Incoming commands");
    let mut cells = vec![ClientIdentitySlot::default(); 2];
    let mut identities =
        ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut cells).expect("Identities");
    seed(&mut identities, 9);
    let mut work = vec![ClientIdentitySlot::default(); 2];
    let pending_commands = server.prepare(&plan).expect("Command resources");
    let pending_identities = identities
        .prepare_service_plan(
            &plan,
            ClientIdentityBudget {
                max_clients: 2,
                max_records: 3,
            },
            &mut work,
        )
        .expect("Identity resources");
    drop(pending_identities);
    drop(pending_commands);
    assert_eq!(server.newest_sequence(), 0);
    assert_eq!(identities.accepted_sequence(), Some(9));
    assert!(identities.is_empty());
    let pending_commands = server.prepare(&plan).expect("Retry commands");
    let pending_identities = identities
        .prepare_service_plan(
            &plan,
            ClientIdentityBudget {
                max_clients: 2,
                max_records: 3,
            },
            &mut work,
        )
        .expect("Retry identities");
    let commands = pending_commands.commit();
    assert_eq!(commands.new_commands, 2);
    output.bytes.fill(0);
    output.updates.fill(ClientDataUpdateSlot::default());
    let identities_change = pending_identities.commit();
    work.fill(ClientIdentitySlot::default());
    assert_eq!(
        (
            identities_change.accepted_records,
            identities_change.ignored_records
        ),
        (2, 1)
    );
    assert_eq!(identities.accepted_sequence(), Some(11));
    assert_eq!(
        identities.client(3).expect("Exact successor").name(),
        b"Player."
    );
    assert_eq!(
        identities.client(63).expect("Raw name").name(),
        b"Other\xff"
    );
    assert_eq!(
        server.command(1).expect("Retained command").as_bytes(),
        b"Private.Command"
    );
    assert_eq!(server.command(2).expect("Empty command").as_bytes(), b"");
}

#[test]
fn late_identity_preflight_rejection_cancels_already_prepared_command_retention() {
    let mut fixture = Fixture::new(&mixed(), 0, b"");
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let outgoing = LegacyClientCommands::new(&mut raw).expect("Outgoing");
    let payload = fixture.decode(&outgoing);
    let mut output = Output::new(64, 8);
    let plan = LegacyServicePlan::parse_with_client_data(
        &payload,
        BUDGET,
        &mut [0; 64],
        output.storage(&mut vec![ClientDataUpdateSlot::default(); 8]),
    )
    .expect("Whole message");
    let mut server_bytes = vec![0; 2 * SERVER_COMMAND_SLOT_BYTES];
    let mut server_slots = [ServerCommandSlot::default(); 2];
    let mut server = ServerCommandHistory::new(
        BaselineProtocol::Cod4x21,
        &mut server_bytes,
        &mut server_slots,
    )
    .expect("Server history");
    let mut cells = vec![ClientIdentitySlot::default(); 1];
    let mut identities =
        ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut cells).expect("Identities");
    seed(&mut identities, 9);
    let pending = server.prepare(&plan).expect("Commands staged first");
    assert_eq!(
        identities
            .prepare_service_plan(
                &plan,
                ClientIdentityBudget {
                    max_clients: 1,
                    max_records: 3
                },
                &mut vec![ClientIdentitySlot::default(); 2]
            )
            .err(),
        Some(ClientIdentityError::ClientBudget)
    );
    drop(pending);
    assert_eq!(server.newest_sequence(), 0);
    assert_eq!(server.last_message_sequence(), 0);
    assert!(identities.is_empty());
    assert_eq!(identities.accepted_sequence(), Some(9));
}

#[test]
fn whole_plan_application_requires_the_initialized_original_profile() {
    let mut fixture = Fixture::new(&mixed(), 0, b"");
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let outgoing = LegacyClientCommands::new(&mut raw).expect("History");
    let payload = fixture.decode(&outgoing);
    let mut output = Output::new(64, 8);
    let plan = LegacyServicePlan::parse_with_client_data(
        &payload,
        BUDGET,
        &mut [0; 64],
        output.storage(&mut vec![ClientDataUpdateSlot::default(); 8]),
    )
    .expect("Whole message");
    let mut cells = vec![ClientIdentitySlot::default(); 2];
    let mut identities =
        ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut cells).expect("Owner");
    assert_eq!(
        identities
            .prepare_service_plan(
                &plan,
                ClientIdentityBudget {
                    max_clients: 2,
                    max_records: 3
                },
                &mut vec![ClientIdentitySlot::default(); 2]
            )
            .err(),
        Some(ClientIdentityError::MissingBaseline)
    );
    seed(&mut identities, 9);
    let mut legacy = Fixture::new(&[0, 7], 0, b"");
    let payload = legacy.decode_profile(&outgoing, BaselineProtocol::Legacy6);
    let plan =
        LegacyServicePlan::parse(&payload, ServiceControlBudget::default(), &mut [], &mut [])
            .expect("Legacy control plan");
    assert_eq!(
        identities
            .prepare_service_plan(
                &plan,
                ClientIdentityBudget {
                    max_clients: 2,
                    max_records: 3
                },
                &mut vec![ClientIdentitySlot::default(); 2]
            )
            .err(),
        Some(ClientIdentityError::Profile)
    );
    assert_eq!(identities.accepted_sequence(), Some(9));
}
