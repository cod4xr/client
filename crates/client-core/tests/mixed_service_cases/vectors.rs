// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn mixed_control_plan_retains_commands_and_complete_normalized_owned_updates() {
    let source = mixed();
    let mut fixture = Fixture::new(&source, 0, b"");
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut raw).expect("History");
    let payload = fixture.decode(&history);
    let mut output = Output::new(64, 8);
    let mut work = vec![ClientDataUpdateSlot::default(); 8];
    let plan = LegacyServicePlan::parse_with_client_data(
        &payload,
        BUDGET,
        &mut [0; 64],
        output.storage(&mut work),
    )
    .expect("Whole mixed controls");
    assert_eq!(plan.profile(), BaselineProtocol::Cod4x21);
    assert_eq!(plan.message_sequence(), 7);
    assert_eq!(plan.client_command_acknowledge(), 0);
    assert_eq!(plan.record_count(), 8);
    assert_eq!(plan.logical_bytes(), source.len());
    assert_eq!(plan.command_count(), 2);
    assert_eq!(
        plan.command(0).expect("Command").as_bytes(),
        b"Private.Command"
    );
    assert_eq!(plan.command(1).expect("Empty").as_bytes(), b"");
    assert_eq!(plan.client_update_count(), 3);
    assert_eq!(
        plan.client_updates()
            .map(ClientDataUpdate::sequence)
            .collect::<Vec<_>>(),
        [10, 12, 11]
    );
    let first = plan.client_updates().next().expect("Identity");
    assert_eq!(first.data().index(), 3);
    assert_eq!(first.data().name(), b"Player.");
    assert_eq!(first.data().clan_tag(), b"Clan.");
    assert!(!format!("{plan:?}").contains("Private"));
}

#[test]
fn whole_plan_outlives_input_history_and_both_private_work_buffers() {
    let mut output = Output::new(64, 8);
    let plan = {
        let mut fixture = Fixture::new(&mixed(), 0, b"");
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let history = LegacyClientCommands::new(&mut raw).expect("History");
        let payload = fixture.decode(&history);
        let mut private = vec![ClientDataUpdateSlot::default(); 8];
        LegacyServicePlan::parse_with_client_data(
            &payload,
            BUDGET,
            &mut [0; 64],
            output.storage(&mut private),
        )
        .expect("Cooked plan")
    };
    assert_eq!(
        plan.command(0).expect("Retained command").as_bytes(),
        b"Private.Command"
    );
    assert_eq!(
        plan.client_updates()
            .next()
            .expect("Retained update")
            .data()
            .name(),
        b"Player."
    );
}

#[test]
fn identity_count_is_caller_funded_without_inheriting_command_or_player_array_limits() {
    let mut source = Vec::new();
    for sequence in 1..=200 {
        append_update(&mut source, sequence, 3, b"n", b"");
    }
    source.push(7);
    let mut fixture = Fixture::new(&source, 0, b"");
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut raw).expect("History");
    let payload = fixture.decode(&history);
    let mut output = Output::new(0, 200);
    let mut work = vec![ClientDataUpdateSlot::default(); 200];
    let plan = LegacyServicePlan::parse_with_client_data(
        &payload,
        ClientServiceBudget {
            max_records: 201,
            max_client_updates: 200,
        },
        &mut [],
        output.storage(&mut work),
    )
    .expect("200 ordinary identity records");
    assert_eq!(plan.client_update_count(), 200);
    assert_eq!(plan.command_count(), 0);
    let mut cells = vec![ClientIdentitySlot::default(); 1];
    let mut state = ClientIdentityState::new(BaselineProtocol::Cod4x21, &mut cells).expect("Owner");
    seed(&mut state, 0);
    let change = state
        .prepare_service_plan(
            &plan,
            ClientIdentityBudget {
                max_clients: 1,
                max_records: 200,
            },
            &mut [ClientIdentitySlot::default()],
        )
        .expect("Whole-plan application")
        .commit();
    assert_eq!(change.accepted_records, 200);
    assert_eq!(state.accepted_sequence(), Some(200));
    assert_eq!(state.len(), 1);
}

#[test]
fn legacy_entry_point_stays_strict_and_new_entry_point_requires_explicit_profile21() {
    let source = mixed();
    let mut fixture = Fixture::new(&source, 0, b"");
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut raw).expect("History");
    let payload = fixture.decode(&history);
    let mut bytes = [0xa5; 64];
    assert!(matches!(
        LegacyServicePlan::parse(
            &payload,
            ServiceControlBudget::default(),
            &mut [0; 64],
            &mut bytes
        ),
        Err(ServiceControlError::UnsupportedService(11))
    ));
    assert_eq!(bytes, [0xa5; 64]);
    let mut legacy = Fixture::new(&source, 0, b"");
    let payload = legacy.decode_profile(&history, BaselineProtocol::Legacy6);
    let mut output = Output::dirty(64, 8);
    let before = output.clone();
    assert_eq!(
        LegacyServicePlan::parse_with_client_data(
            &payload,
            BUDGET,
            &mut [0; 64],
            output.storage(&mut vec![ClientDataUpdateSlot::default(); 8])
        )
        .err(),
        Some(ServiceControlError::Profile)
    );
    assert_eq!(output, before);
}

#[test]
fn command_only_messages_expose_no_stale_identity_cells_after_reuse() {
    let mut fixture = Fixture::new(&[0, 7], 0, b"");
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut raw).expect("History");
    let payload = fixture.decode(&history);
    let mut output = Output::dirty(64, 8);
    let before = output.clone();
    let plan = LegacyServicePlan::parse_with_client_data(
        &payload,
        ClientServiceBudget {
            max_records: 2,
            max_client_updates: 0,
        },
        &mut [],
        output.storage(&mut []),
    )
    .expect("No identities");
    assert_eq!(plan.client_update_count(), 0);
    assert_eq!(plan.client_updates().count(), 0);
    assert_eq!(output, before);
}
