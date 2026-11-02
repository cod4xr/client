// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::{MessageError, MessageStringLimit};
use cod4xr_client_core::{
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    service_controls::{LegacyServicePlan, ServiceControlBudget, ServiceControlError},
};

use super::{Fixture, append_command};

#[test]
fn truncated_records_and_malformed_suffixes_never_publish_partial_commands() {
    let mut source = Vec::new();
    append_command(&mut source, 1, b"PrivateCommand");
    source.push(7);
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut storage).expect("History");
    for end in 1..source.len() {
        let mut fixture = Fixture::new(&source[..end], 0, b"");
        let payload = fixture.decode(&history);
        let mut output = [0xa5; 64];
        assert!(
            LegacyServicePlan::parse(
                &payload,
                ServiceControlBudget::default(),
                &mut [0xa5; 64],
                &mut output
            )
            .is_err()
        );
        assert_eq!(output, [0xa5; 64]);
    }
    for opcode in [1, 2, 3, 5, 6, 8, 9, 10, 11, 12, 0xfe] {
        let mut malformed = source.clone();
        let last = malformed.len() - 1;
        malformed[last] = opcode;
        let mut fixture = Fixture::new(&malformed, 0, b"");
        let payload = fixture.decode(&history);
        let mut output = [0xa5; 64];
        assert!(
            matches!(LegacyServicePlan::parse(&payload, ServiceControlBudget::default(), &mut [0xa5; 64], &mut output),
            Err(ServiceControlError::UnsupportedService(value)) if value == opcode)
        );
        assert_eq!(output, [0xa5; 64]);
    }
}

#[test]
fn every_short_string_scratch_and_final_output_preserves_public_output() {
    let mut source = Vec::new();
    append_command(&mut source, 1, b"Private%");
    source.push(7);
    let mut fixture = Fixture::new(&source, 0, b"");
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut storage).expect("History");
    let payload = fixture.decode(&history);
    for capacity in 0..b"Private%".len() {
        let mut output = [0xa5; 32];
        assert!(matches!(
            LegacyServicePlan::parse(
                &payload,
                ServiceControlBudget::default(),
                &mut [0xa5; 32][..capacity],
                &mut output
            ),
            Err(ServiceControlError::Message(MessageError::Capacity { .. }))
        ));
        assert_eq!(output, [0xa5; 32]);
        assert!(matches!(
            LegacyServicePlan::parse(
                &payload,
                ServiceControlBudget::default(),
                &mut [0xa5; 32],
                &mut output[..capacity]
            ),
            Err(ServiceControlError::OutputCapacity)
        ));
        assert_eq!(output, [0xa5; 32]);
    }
}

#[test]
fn record_and_command_budgets_require_eof_within_the_admitted_window() {
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut storage).expect("History");
    let mut fixture = Fixture::new(&[0, 7], 0, b"");
    let payload = fixture.decode(&history);
    assert!(matches!(
        LegacyServicePlan::parse(
            &payload,
            ServiceControlBudget { max_records: 0 },
            &mut [],
            &mut []
        ),
        Err(ServiceControlError::InvalidBudget)
    ));
    assert!(matches!(
        LegacyServicePlan::parse(
            &payload,
            ServiceControlBudget { max_records: 1 },
            &mut [],
            &mut []
        ),
        Err(ServiceControlError::RecordBudget)
    ));
    assert!(
        LegacyServicePlan::parse(
            &payload,
            ServiceControlBudget { max_records: 2 },
            &mut [],
            &mut []
        )
        .is_ok()
    );
    let mut source = Vec::new();
    for sequence in 1..=129 {
        append_command(&mut source, sequence, b"x");
    }
    source.push(7);
    let mut too_many = Fixture::new(&source, 0, b"");
    let payload = too_many.decode(&history);
    let mut output = [0xa5; 256];
    assert!(matches!(
        LegacyServicePlan::parse(
            &payload,
            ServiceControlBudget::default(),
            &mut [0xa5; 8],
            &mut output
        ),
        Err(ServiceControlError::CommandBudget)
    ));
    assert_eq!(output, [0xa5; 256]);
}

#[test]
fn missing_eof_sequence_domains_and_big_string_overflow_reject_without_partial_output() {
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut storage).expect("History");
    // Eight 3-bit NOP Huffman codes end exactly at a byte boundary: no padding symbol.
    let mut fixture = Fixture::new(&[0; 8], 0, b"");
    let payload = fixture.decode(&history);
    assert!(matches!(
        LegacyServicePlan::parse(&payload, ServiceControlBudget::default(), &mut [], &mut []),
        Err(ServiceControlError::MissingEof)
    ));
    for sequence in [i32::MIN, -1, 0] {
        let mut source = Vec::new();
        append_command(&mut source, sequence, b"PrivateCommand");
        source.push(7);
        let mut fixture = Fixture::new(&source, 0, b"");
        let payload = fixture.decode(&history);
        let mut output = [0xa5; 64];
        assert!(matches!(
            LegacyServicePlan::parse(
                &payload,
                ServiceControlBudget::default(),
                &mut [0xa5; 64],
                &mut output
            ),
            Err(ServiceControlError::Sequence)
        ));
        assert_eq!(output, [0xa5; 64]);
    }
    let mut source = Vec::new();
    let too_long = vec![b'x'; MessageStringLimit::Big.maximum_bytes() + 1];
    append_command(&mut source, 1, &too_long);
    source.push(7);
    let mut fixture = Fixture::new(&source, 0, b"");
    let payload = fixture.decode(&history);
    let mut scratch = vec![0xa5; too_long.len()];
    let mut output = vec![0xa5; too_long.len()];
    assert!(matches!(
        LegacyServicePlan::parse(
            &payload,
            ServiceControlBudget::default(),
            &mut scratch,
            &mut output
        ),
        Err(ServiceControlError::Message(
            MessageError::StringTooLong { .. }
        ))
    ));
    assert!(output.iter().all(|&byte| byte == 0xa5));
}
