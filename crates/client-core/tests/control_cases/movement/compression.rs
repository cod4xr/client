// SPDX-License-Identifier: AGPL-3.0-or-later
use super::support::{body, commands, header, queued};
use crate::control_messages::CHALLENGE;
use cod4x_protocol::{
    MessageReader,
    user_command::{self, LegacyUserCommand},
};
use cod4xr_client_core::{
    BaselineProtocol,
    control::{ControlMessagePlan, MovementBatch},
    player_state::{PredictionSample, PredictionWords},
    user_commands::{MovementAxis, UserCommandMelee},
};

#[test]
fn compression_expansion_defers_last_reliable_command_and_preserves_the_whole_movement_batch() {
    let profile = BaselineProtocol::Cod4x21;
    let mut input = commands(32);
    for command in &mut input {
        command.server_time = i32::from_le_bytes([0xd5; 4]);
        command.buttons = 0x10004;
        command.angles = [0xd5d5, 0xd5d5, 0];
        command.weapon = 0;
        command.offhand = 0;
        command.movement = [MovementAxis::Stationary; 2];
        command.selected_location = Some([0; 2]);
        command.melee = Some(UserCommandMelee {
            yaw: 0,
            distance: 0,
        });
    }
    // D5 costs eleven Huffman bits. The complete plaintext fits the private frame,
    // while this reliable string plus the batch exceeds its compressed capacity.
    let reliable = [0xd5; 1023];
    let origin = [0xd5d5_d5d5; 3];
    let report = PredictionSample::new(
        profile,
        0xd5d5_d5d5,
        PredictionWords::new(origin, [0; 3], [0; 3], 0, 0),
    );
    let frame = queued(&[&reliable], |history| {
        let idle =
            ControlMessagePlan::new(header(profile, 7, b"Private.After"), CHALLENGE, history)
                .expect("Reliable command fits alone");
        assert_eq!(idle.command_count(), 1);
        let frame = ControlMessagePlan::with_movement(
            header(profile, 7, b"Private.After"),
            CHALLENGE,
            history,
            MovementBatch {
                commands: &input,
                checksum_feed: 7 ^ 0x0002_6e0b,
                prediction: report,
                snapshot: None,
            },
        )
        .expect("Compression retry with movement intact");
        assert_eq!(
            (
                history.pending_count(),
                history.transmitted_sequence(),
                history.acknowledged_sequence()
            ),
            (1, 0, 0)
        );
        frame
    });
    assert_eq!(
        (
            frame.command_count(),
            frame.user_command_count(),
            frame.last_command_sequence()
        ),
        (0, 32, None)
    );
    let decoded = body(&frame, profile, 7, b"Private.After");
    let mut reader = MessageReader::try_new(&decoded).expect("Complete movement-only retry");
    assert_eq!(
        reader
            .read_bits(3)
            .expect("Reliable deferred before movement"),
        1
    );
    assert_eq!(reader.read_u8().expect("No dropped inputs"), 32);
    let mut prior = LegacyUserCommand::default();
    for expected in input {
        prior = user_command::read_delta(&mut reader, 0, &prior)
            .expect("Every complete native command");
        assert_eq!(
            (prior.server_time, prior.buttons, prior.angles),
            (expected.server_time, expected.buttons, expected.angles)
        );
    }
    for expected in origin {
        assert_eq!(
            reader.read_i32().expect("Origin report").cast_unsigned(),
            expected
        );
    }
    assert_eq!(
        reader.read_i32().expect("Prediction time").cast_unsigned(),
        0xd5d5_d5d5
    );
    assert_eq!(
        reader.read_bits(3).expect("Complete EOF after all inputs"),
        3
    );
}
