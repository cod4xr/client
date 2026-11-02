// SPDX-License-Identifier: AGPL-3.0-or-later
use super::support::{FEED, build, commands, header, prediction, queued, snapshot};
use crate::control_messages::CHALLENGE;
use cod4xr_client_core::{
    BaselineProtocol,
    control::{ControlMessageError, ControlMessagePlan, MovementBatch, MovementMessageError},
    user_commands::UserCommandEncodeError,
};

#[test]
fn invalid_batch_count_profile_and_last_command_fields_reject_without_transmission_admission() {
    let profile = BaselineProtocol::Cod4x21;
    queued(&[b"pending"], |history| {
        for mode in 0..4 {
            let mut input = commands(if mode == 0 {
                0
            } else if mode == 1 {
                33
            } else {
                32
            });
            let report = prediction(if mode == 2 {
                BaselineProtocol::Legacy6
            } else {
                profile
            });
            if mode == 3 {
                input.last_mut().expect("Last record").weapon = 128;
            }
            let expected = match mode {
                0 | 1 => MovementMessageError::CommandCount,
                2 => MovementMessageError::PredictionProfile,
                _ => MovementMessageError::Command(UserCommandEncodeError::Weapon),
            };
            assert_eq!(
                ControlMessagePlan::with_movement(
                    header(profile, 7, b"Private.After"),
                    CHALLENGE,
                    history,
                    MovementBatch {
                        commands: &input,
                        checksum_feed: FEED,
                        prediction: report,
                        snapshot: None
                    }
                )
                .expect_err("Complete movement validation"),
                ControlMessageError::Movement(expected)
            );
            assert_eq!(
                (
                    history.acknowledged_sequence(),
                    history.transmitted_sequence(),
                    history.pending_count()
                ),
                (0, 0, 1)
            );
        }
    });
}
#[test]
fn wrong_profile_and_future_snapshot_references_reject_before_owned_frame_publication() {
    let profile = BaselineProtocol::Cod4x21;
    let input = commands(1);
    for (source_profile, sequence, expected) in [
        (
            BaselineProtocol::Legacy6,
            7,
            MovementMessageError::SnapshotProfile,
        ),
        (profile, 8, MovementMessageError::SnapshotSequence),
    ] {
        snapshot(source_profile, sequence, |source| {
            queued(&[], |history| {
                assert_eq!(
                    ControlMessagePlan::with_movement(
                        header(profile, 7, b"Private.After"),
                        CHALLENGE,
                        history,
                        MovementBatch {
                            commands: &input,
                            checksum_feed: FEED,
                            prediction: prediction(profile),
                            snapshot: Some(source)
                        }
                    )
                    .expect_err("Reference binding"),
                    ControlMessageError::Movement(expected)
                );
            });
        });
    }
}
#[test]
fn bad_header_precedes_movement_validation_and_owned_output_remains_atomic_and_private() {
    let profile = BaselineProtocol::Cod4x21;
    queued(&[], |history| {
        assert_eq!(
            ControlMessagePlan::with_movement(
                header(profile, 0x8000_0000, b"Private.After"),
                CHALLENGE,
                history,
                MovementBatch {
                    commands: &[],
                    checksum_feed: FEED,
                    prediction: prediction(profile),
                    snapshot: None
                }
            )
            .expect_err("Header before empty batch"),
            ControlMessageError::Header
        );
    });
    let input = commands(2);
    let frame = build(profile, 7, b"Private.After", &input, &[], None);
    for capacity in [0, frame.encoded_len() - 1] {
        let mut bytes = [0xa5; 2048];
        assert_eq!(
            frame.encode(&mut bytes[..capacity]),
            Err(ControlMessageError::OutputCapacity)
        );
        assert_eq!(bytes, [0xa5; 2048]);
    }
    let batch = MovementBatch {
        commands: &input,
        checksum_feed: FEED,
        prediction: prediction(profile),
        snapshot: None,
    };
    let text = format!("{batch:?} {frame:?}");
    for private in ["Private", "2271560481", "4276996862", "2147483648"] {
        assert!(!text.contains(private));
    }
    assert!(!text.contains("checksum_feed"));
}
