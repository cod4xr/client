// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::{MAX_COMMAND_BYTES, MessageStringLimit};
use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    service_controls::{LegacyServicePlan, ServiceControlBudget},
};

use crate::{Fixture, append_command};

#[test]
fn nop_and_eof_validate_metadata_without_interpreting_padding_or_following_records() {
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut storage).expect("History");
    let mut fixture = Fixture::new(&[0, 7, 0xfe, 4, 99, 0, 0], 0, b"");
    let payload = fixture.decode(&history);
    let mut output = [0xa5; 8];
    let plan = LegacyServicePlan::parse(
        &payload,
        ServiceControlBudget { max_records: 2 },
        &mut [],
        &mut output,
    )
    .expect("Logical stream ends before physical padding/following bytes");
    assert_eq!(plan.profile(), BaselineProtocol::Cod4x21);
    assert_eq!(plan.message_sequence(), 7);
    assert_eq!(plan.client_command_acknowledge(), 0);
    assert_eq!(
        (
            plan.record_count(),
            plan.logical_bytes(),
            plan.command_count(),
            plan.command_bytes()
        ),
        (2, 2, 0, 0)
    );
    assert!(plan.command(0).is_none());
    assert!(plan.command(usize::MAX).is_none());
    assert_eq!(plan.commands().len(), 0);
    assert_eq!(output, [0xa5; 8]);
}

#[test]
fn complete_records_preserve_order_duplicates_empty_strings_and_native_byte_normalization() {
    let mut source = Vec::new();
    append_command(&mut source, 7, b"PrivateCommand%\x92\xff");
    source.push(0);
    append_command(&mut source, 5, b"");
    append_command(&mut source, 7, b"duplicate sequence");
    source.push(7);
    let mut fixture = Fixture::new(&source, 0, b"");
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut storage).expect("History");
    let payload = fixture.decode(&history);
    let mut scratch = [0xa5; 128];
    let mut output = [0xa5; 128];
    {
        let plan = LegacyServicePlan::parse(
            &payload,
            ServiceControlBudget::default(),
            &mut scratch,
            &mut output,
        )
        .expect("Complete control records");
        assert_eq!(
            (
                plan.record_count(),
                plan.logical_bytes(),
                plan.command_count()
            ),
            (5, source.len(), 3)
        );
        let commands: Vec<_> = plan
            .commands()
            .map(|command| (command.sequence(), command.as_bytes().to_vec()))
            .collect();
        assert_eq!(
            commands,
            [
                (7, b"PrivateCommand.\x92\xff".to_vec()),
                (5, Vec::new()),
                (7, b"duplicate sequence".to_vec())
            ]
        );
        assert_eq!(
            plan.command(1)
                .expect("Empty record")
                .key()
                .expect("Empty retained key")
                .as_bytes(),
            b""
        );
        assert!(plan.command(3).is_none());
        assert!(!format!("{plan:?}").contains("PrivateCommand"));
        assert!(
            !format!("{:?}", plan.command(0).expect("First command")).contains("PrivateCommand")
        );
    }
    let used = b"PrivateCommand.\x92\xff".len() + b"duplicate sequence".len();
    assert!(output[used..].iter().all(|&byte| byte == 0xa5));
    assert_eq!(history.acknowledged_sequence(), 0);
}

#[test]
fn maximum_big_command_is_complete_while_its_retained_key_uses_only_the_native_prefix() {
    let mut text = vec![b'x'; MessageStringLimit::Big.maximum_bytes()];
    text[0] = 0x92;
    text[MAX_COMMAND_BYTES - 1] = b'%';
    text[MAX_COMMAND_BYTES] = b'%';
    let last = text.len() - 1;
    text[last] = b'%';
    let mut source = Vec::new();
    append_command(&mut source, i32::MAX, &text);
    source.push(7);
    let mut fixture = Fixture::new(&source, 0, b"");
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut storage).expect("History");
    let payload = fixture.decode(&history);
    let mut scratch = vec![0xa5; text.len()];
    let mut output = vec![0xa5; text.len() + 1];
    {
        let plan = LegacyServicePlan::parse(
            &payload,
            ServiceControlBudget::default(),
            &mut scratch,
            &mut output,
        )
        .expect("Maximum command");
        let command = plan.command(0).expect("Complete record");
        assert_eq!(command.as_bytes().len(), text.len());
        assert_eq!(
            command.sequence(),
            u32::try_from(i32::MAX).expect("Positive maximum")
        );
        assert_eq!(command.as_bytes()[0], 0x92);
        assert_eq!(command.as_bytes()[MAX_COMMAND_BYTES], b'.');
        assert_eq!(command.as_bytes()[last], b'.');
        let key = command.key().expect("Key prefix");
        assert_eq!(key.as_bytes().len(), MAX_COMMAND_BYTES);
        assert_eq!(key.as_bytes()[MAX_COMMAND_BYTES - 1], b'.');
    }
    assert_eq!(output[text.len()], 0xa5);
}

#[test]
fn validated_plan_outlives_decoded_input_and_does_not_commit_acknowledgement_itself() {
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("History");
    history.enqueue(b"a%").expect("Canonical key");
    history.mark_transmitted(1).expect("Sent");
    let mut output = [0xa5; 64];
    let plan = {
        let mut source = Vec::new();
        append_command(&mut source, 3, b"PrivateOwnedCommand");
        source.push(7);
        let mut fixture = Fixture::new(&source, 1, b"a.");
        let payload = fixture.decode(&history);
        LegacyServicePlan::parse(
            &payload,
            ServiceControlBudget::default(),
            &mut [0xa5; 64],
            &mut output,
        )
        .expect("Owned normalized plan")
    };
    assert_eq!(history.acknowledged_sequence(), 0);
    assert_eq!(plan.client_command_acknowledge(), 1);
    assert_eq!(
        plan.command(0).expect("Retained output").as_bytes(),
        b"PrivateOwnedCommand"
    );
    history
        .commit_acknowledgement(plan.client_command_acknowledge())
        .expect("Same-connection host commit after validation");
    assert_eq!(history.pending_count(), 0);
}

#[test]
fn complete_baseline_window_retains_every_record_and_admits_its_final_eof_slot() {
    let mut source = Vec::new();
    for sequence in 1..=128 {
        append_command(&mut source, sequence, b"x");
    }
    source.push(7);
    let mut fixture = Fixture::new(&source, 0, b"");
    let mut storage = [0xa5; 2 * LEGACY_COMMAND_SLOT_BYTES];
    let history = LegacyClientCommands::new(&mut storage).expect("History");
    let payload = fixture.decode(&history);
    let mut output = [0xa5; 128];
    let plan = LegacyServicePlan::parse(
        &payload,
        ServiceControlBudget { max_records: 129 },
        &mut [0; 1],
        &mut output,
    )
    .expect("Full window and EOF");
    assert_eq!(
        (
            plan.command_count(),
            plan.record_count(),
            plan.command_bytes()
        ),
        (128, 129, 128)
    );
    assert_eq!(
        plan.commands()
            .map(|command| command.sequence())
            .collect::<Vec<_>>(),
        (1..=128).collect::<Vec<_>>()
    );
    assert!(plan.commands().all(|command| command.as_bytes() == b"x"));
    assert_eq!(plan.command(127).expect("Final record").sequence(), 128);
    assert!(plan.command(128).is_none());
}
