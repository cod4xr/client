// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn every_truncation_and_late_unsupported_service_preserve_all_final_cells_and_text() {
    let source = mixed();
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut raw).expect("History");
    for end in 1..source.len() {
        let mut fixture = Fixture::new(&source[..end], 0, b"");
        let payload = fixture.decode(&history);
        let mut output = Output::dirty(64, 8);
        let before = output.clone();
        assert!(
            LegacyServicePlan::parse_with_client_data(
                &payload,
                BUDGET,
                &mut [0; 64],
                output.storage(&mut vec![ClientDataUpdateSlot::default(); 8])
            )
            .is_err()
        );
        assert_eq!(output, before);
    }
    for opcode in [1, 2, 3, 5, 6, 8, 9, 10, 12, 255] {
        let mut source = source.clone();
        let end = source.len() - 1;
        source[end] = opcode;
        let mut fixture = Fixture::new(&source, 0, b"");
        let payload = fixture.decode(&history);
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
            Some(ServiceControlError::UnsupportedService(opcode))
        );
        assert_eq!(output, before);
    }
}

#[test]
fn record_identity_private_and_final_capacities_are_independent_and_atomic() {
    let mut fixture = Fixture::new(&mixed(), 0, b"");
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut raw).expect("History");
    let payload = fixture.decode(&history);
    for (budget, private, finals, bytes, error) in [
        (
            ClientServiceBudget {
                max_records: 7,
                ..BUDGET
            },
            8,
            8,
            64,
            ServiceControlError::RecordBudget,
        ),
        (
            ClientServiceBudget {
                max_client_updates: 2,
                ..BUDGET
            },
            8,
            8,
            64,
            ServiceControlError::ClientDataBudget,
        ),
        (BUDGET, 2, 8, 64, ServiceControlError::ClientDataScratch),
        (BUDGET, 8, 2, 64, ServiceControlError::ClientDataCapacity),
        (BUDGET, 8, 8, 14, ServiceControlError::OutputCapacity),
    ] {
        let mut output = Output::dirty(bytes, finals);
        let before = output.clone();
        assert_eq!(
            LegacyServicePlan::parse_with_client_data(
                &payload,
                budget,
                &mut [0; 64],
                output.storage(&mut vec![ClientDataUpdateSlot::default(); private])
            )
            .err(),
            Some(error)
        );
        assert_eq!(output, before);
    }
}

#[test]
fn malformed_client_identity_after_a_valid_command_never_publishes_either_output() {
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut raw).expect("History");
    for (index, name, tag) in [
        (64, b"ok".as_slice(), b"".as_slice()),
        (3, &[b'x'; 33], b""),
        (3, b"ok", &[b'x'; 13]),
    ] {
        let mut source = Vec::new();
        append_command(&mut source, 1, b"Private%");
        append_update(&mut source, 10, index, name, tag);
        source.push(7);
        let mut fixture = Fixture::new(&source, 0, b"");
        let payload = fixture.decode(&history);
        let mut output = Output::dirty(64, 8);
        let before = output.clone();
        assert!(matches!(
            LegacyServicePlan::parse_with_client_data(
                &payload,
                BUDGET,
                &mut [0; 64],
                output.storage(&mut vec![ClientDataUpdateSlot::default(); 8])
            ),
            Err(ServiceControlError::ClientData(_))
        ));
        assert_eq!(output, before);
    }
}

#[test]
fn logical_eof_keeps_native_padding_and_following_bytes_out_of_the_plan() {
    let mut source = mixed();
    let logical = source.len();
    source.extend_from_slice(&[11, 255, 255, 255, 255, 255]);
    let mut fixture = Fixture::new(&source, 0, b"");
    let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut raw).expect("History");
    let payload = fixture.decode(&history);
    let mut output = Output::new(64, 8);
    let plan = LegacyServicePlan::parse_with_client_data(
        &payload,
        BUDGET,
        &mut [0; 64],
        output.storage(&mut vec![ClientDataUpdateSlot::default(); 8]),
    )
    .expect("Logical EOF");
    assert_eq!(plan.logical_bytes(), logical);
    assert_eq!(plan.client_update_count(), 3);
}
