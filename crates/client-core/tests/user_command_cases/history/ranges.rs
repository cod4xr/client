// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::user_commands::zero;
use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    control::{ClientControlHeader, ControlMessagePlan, ServerCommandKey},
    player_state::{PredictionSample, PredictionWords},
    user_commands::history::{InputCommandHistory, InputCommandSlot, InputHistoryError},
};

#[test]
fn cancellation_backup_only_and_overlapping_new_prefix_have_distinct_transmission_effects() {
    let mut cells = vec![InputCommandSlot::default(); 8];
    let mut owner =
        InputCommandHistory::new(7, BaselineProtocol::Cod4x21, &mut cells).expect("Owner");
    let ids: Vec<_> = (0..5)
        .map(|_| owner.enqueue(zero()).expect("Input"))
        .collect();
    let pending = owner
        .prepare_range(ids[0], ids[2])
        .expect("Prepared original range");
    assert_eq!(
        (
            pending.summary().command_count,
            pending.summary().newly_transmitted
        ),
        (3, 3)
    );
    drop(pending);
    assert_eq!((owner.pending_count(), owner.transmitted_id()), (5, None));
    let _ = owner
        .prepare_range(ids[0], ids[2])
        .expect("Retry same input")
        .commit_transmission();
    let repeated = owner
        .prepare_range(ids[0], ids[1])
        .expect("Backup-only send")
        .commit_transmission();
    assert_eq!(
        (repeated.last, repeated.through, repeated.newly_transmitted),
        (ids[1], ids[2], 0)
    );
    let overlapping = owner
        .prepare_range(ids[1], ids[3])
        .expect("Backup plus next new input")
        .commit_transmission();
    assert_eq!(
        (overlapping.command_count, overlapping.newly_transmitted),
        (3, 1)
    );
    assert_eq!(
        (
            owner.pending_count(),
            owner.transmitted_id(),
            owner.retired_id()
        ),
        (1, Some(ids[3]), None)
    );
    assert_eq!(owner.len(), 5);
}
#[test]
fn range_errors_never_skip_unsent_input_or_clamp_a_native_packet() {
    let mut cells = vec![InputCommandSlot::default(); 40];
    let mut owner =
        InputCommandHistory::new(7, BaselineProtocol::Cod4x21, &mut cells).expect("Owner");
    let ids: Vec<_> = (0..40)
        .map(|_| owner.enqueue(zero()).expect("Input"))
        .collect();
    for (first, last, expected) in [
        (ids[1], ids[2], InputHistoryError::TransmissionGap),
        (ids[2], ids[1], InputHistoryError::BatchRange),
        (ids[0], ids[32], InputHistoryError::BatchCount),
    ] {
        assert_eq!(
            owner
                .prepare_range(first, last)
                .expect_err("Whole range validation"),
            expected
        );
        assert_eq!((owner.pending_count(), owner.transmitted_id()), (40, None));
    }
    let _ = owner
        .prepare_range(ids[0], ids[31])
        .expect("Full native packet")
        .commit_transmission();
    owner.retire_through(ids[15]).expect("Explicit retirement");
    assert_eq!(
        owner
            .prepare_range(ids[0], ids[1])
            .expect_err("Retired range"),
        InputHistoryError::Missing
    );
    assert_eq!(
        owner
            .prepare_range(ids[33], ids[34])
            .expect_err("Cannot skip first pending input"),
        InputHistoryError::TransmissionGap
    );
}
#[test]
fn owned_packet_outlives_prepared_copy_and_input_storage_without_admitting_a_cancelled_send() {
    let frame = {
        let mut cells = vec![InputCommandSlot::default(); 4];
        let mut owner =
            InputCommandHistory::new(7, BaselineProtocol::Cod4x21, &mut cells).expect("Owner");
        let id = owner.enqueue(zero()).expect("Input");
        let pending = owner.prepare_range(id, id).expect("Original owner");
        let report = PredictionSample::new(
            BaselineProtocol::Cod4x21,
            0,
            PredictionWords::new([0; 3], [0; 3], [0; 3], 0, 0),
        );
        let movement = pending
            .movement(0, report, None)
            .expect("Same input profile");
        let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
        let reliable = LegacyClientCommands::new(&mut raw).expect("Reliable owner");
        let header = ClientControlHeader::Cod4x21 {
            server_id: 0,
            message_acknowledge: 0,
            server_command: ServerCommandKey::initial(),
            config_acknowledge: 0,
        };
        let frame = ControlMessagePlan::with_movement(header, 123, &reliable, movement)
            .expect("Owned whole packet");
        drop(pending);
        assert_eq!((owner.pending_count(), owner.transmitted_id()), (1, None));
        owner
            .reset(8, BaselineProtocol::Legacy6)
            .expect("Explicit world abort");
        frame
    };
    assert_eq!(frame.user_command_count(), 1);
    assert!(!frame.as_bytes().is_empty());
}
