// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, SnapshotBaselines, SnapshotError, SnapshotMessage, SnapshotPlan, body, contexts,
    dirty, input, output, position,
};

#[test]
fn message_sequences_use_shared_ordinary_bounds_and_preserve_command_bits() {
    for sequence in [0, 0x8000_0000, u32::MAX] {
        assert_eq!(
            SnapshotMessage::new(u64::MAX, sequence, 0),
            Err(SnapshotError::MessageSequence)
        );
    }
    for sequence in [1, 255, 256, 0x7fff_ffff] {
        let message = SnapshotMessage::new(u64::MAX, sequence, u32::MAX).expect("Shared sequence");
        assert_eq!(message.epoch(), u64::MAX);
        assert_eq!(message.sequence(), sequence);
        assert_eq!(message.server_command_sequence(), u32::MAX);
    }
}
#[test]
fn baseline_binding_failures_reject_before_wire_reads_or_final_output_changes() {
    let contexts = contexts();
    for (index, &context) in contexts.iter().enumerate() {
        for profile_error in [false, true] {
            let mut selected = input(context, 1, None);
            selected.baselines = SnapshotBaselines::new(
                if profile_error { 7 } else { 8 },
                if profile_error {
                    contexts[1 - index].profile()
                } else {
                    context.profile()
                },
                super::EntityStateSource::from_states(&[]),
            );
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            let mut reader = BodyReader::try_new(&[], 0).expect("Empty input");
            let error =
                SnapshotPlan::prepare(&mut reader, selected, output(&mut entities, &mut clients))
                    .expect_err("Binding error");
            assert_eq!(
                error,
                if profile_error {
                    SnapshotError::BaselineProfile
                } else {
                    SnapshotError::BaselineEpoch
                }
            );
            assert_eq!((entities, clients), saved);
            assert_eq!(reader.byte_position(), 0);
        }
    }
}
#[test]
fn prior_world_profile_and_exact_message_distance_are_required_without_publication() {
    let contexts = contexts();
    let bytes = body(0, true, &[], &[]);
    let mut old_reader = BodyReader::try_new(&bytes, bytes.len()).expect("Old body");
    let (mut old_entities, mut old_clients) = dirty();
    let old = SnapshotPlan::prepare(
        &mut old_reader,
        input(contexts[0], 10, None),
        output(&mut old_entities, &mut old_clients),
    )
    .expect("Old state")
    .commit_transmitted()
    .expect("Old commit");
    for (context, epoch, distance, sequence, expected) in [
        (contexts[0], 8, 1, 11, SnapshotError::ReferenceEpoch),
        (contexts[1], 7, 1, 11, SnapshotError::ReferenceProfile),
        (contexts[0], 7, 2, 11, SnapshotError::ReferenceSequence),
        (contexts[0], 7, 255, 1, SnapshotError::ReferenceSequence),
        (contexts[0], 7, 0, 11, SnapshotError::UnexpectedReference),
    ] {
        let bytes = body(distance, true, &[], &[]);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let before = position(&reader);
        let (mut entities, mut clients) = dirty();
        let saved = (entities.clone(), clients.clone());
        let mut selected = input(context, sequence, Some(old.as_source()));
        selected.message = SnapshotMessage::new(epoch, sequence, 0).expect("Message");
        selected.baselines = SnapshotBaselines::new(
            epoch,
            context.profile(),
            super::EntityStateSource::from_states(&[]),
        );
        assert_eq!(
            SnapshotPlan::prepare(&mut reader, selected, output(&mut entities, &mut clients))
                .expect_err("Binding failure"),
            expected
        );
        assert_eq!(position(&reader), before);
        assert_eq!((entities, clients), saved);
    }
}
#[test]
fn nonzero_distance_never_silently_substitutes_zero_without_a_cooked_prior() {
    for context in contexts() {
        let bytes = body(255, true, &[], &[]);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let (mut entities, mut clients) = dirty();
        let saved = (entities.clone(), clients.clone());
        assert_eq!(
            SnapshotPlan::prepare(
                &mut reader,
                input(context, 256, None),
                output(&mut entities, &mut clients)
            )
            .expect_err("Missing reference"),
            SnapshotError::MissingReference
        );
        assert_eq!(reader.byte_position(), 0);
        assert_eq!((entities, clients), saved);
    }
}
#[test]
fn wire_distance_two_hundred_fifty_five_survives_beyond_native_history_capacity() {
    for context in contexts() {
        let bytes = body(0, true, &[], &[]);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Full");
        let (mut old_entities, mut old_clients) = dirty();
        let old = SnapshotPlan::prepare(
            &mut reader,
            input(context, 1, None),
            output(&mut old_entities, &mut old_clients),
        )
        .expect("Full")
        .commit_transmitted()
        .expect("Commit");
        let bytes = body(255, true, &[], &[]);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Delta");
        let (mut entities, mut clients) = dirty();
        let next = SnapshotPlan::prepare(
            &mut reader,
            input(context, 256, Some(old.as_source())),
            output(&mut entities, &mut clients),
        )
        .expect("Distance 255")
        .commit_transmitted()
        .expect("Commit");
        assert_eq!(next.message().sequence(), 256);
    }
}
