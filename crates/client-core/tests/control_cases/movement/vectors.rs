// SPDX-License-Identifier: AGPL-3.0-or-later
use super::support::{FEED, ORIGIN, PREDICTION_TIME, body, build, commands};
use cod4x_protocol::{
    MessageReader, MessageStringLimit,
    user_command::{self, LegacyUserCommand},
};
use cod4xr_client_core::{BaselineProtocol, control::MAX_PACKET_USER_COMMANDS};

#[test]
fn both_profiles_join_reliable_prefix_full_user_batches_prediction_report_and_eof() {
    for profile in [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21] {
        for count in [1, 2, 32] {
            for (key_bytes, command_hash) in [
                (b"Private.After".as_slice(), 159_243),
                (b"\x92".as_slice(), 4_294_954_194),
            ] {
                let input = commands(count);
                let plan = build(profile, 7, key_bytes, &input, &[b"oldest", b"next"], None);
                assert_eq!(
                    (plan.command_count(), plan.user_command_count()),
                    (2, count)
                );
                assert_eq!(plan.last_command_sequence(), Some(2));
                let decoded = body(&plan, profile, 7, key_bytes);
                let mut reader = MessageReader::try_new(&decoded).expect("Complete payload");
                let mut text = [0; 32];
                for (sequence, expected) in [(1, b"oldest".as_slice()), (2, b"next")] {
                    assert_eq!(reader.read_bits(3).expect("Reliable opcode"), 2);
                    assert_eq!(reader.read_i32().expect("Reliable sequence"), sequence);
                    let length = reader
                        .read_legacy_string(&mut text, MessageStringLimit::Normal)
                        .expect("Reliable text");
                    assert_eq!(&text[..length], expected);
                }
                assert_eq!(reader.read_bits(3).expect("No-delta opcode"), 1);
                assert_eq!(reader.read_u8().expect("Whole native batch count"), count);
                let mut prior = LegacyUserCommand {
                    weapon: 77,
                    offhand: 66,
                    angles: [0xdead; 3],
                    ..LegacyUserCommand::default()
                };
                for input in &input {
                    let command =
                        user_command::read_delta(&mut reader, FEED ^ 7 ^ command_hash, &prior)
                            .expect("Published shared command decoder");
                    assert_eq!(
                        (command.server_time, command.buttons, command.angles),
                        (input.server_time, input.buttons, input.angles)
                    );
                    assert_eq!(
                        (command.weapon, command.offhand),
                        (input.weapon, input.offhand)
                    );
                    assert_eq!((command.forward_move, command.right_move), (-127, 127));
                    assert_eq!(
                        command.selected_location,
                        input.selected_location.expect("Selected")
                    );
                    assert_eq!(command.melee_charge_yaw.to_bits(), (-90.0_f32).to_bits());
                    assert_eq!(
                        command.melee_charge_distance,
                        input.melee.expect("Melee").distance
                    );
                    prior = command;
                }
                for expected in ORIGIN {
                    assert_eq!(
                        reader
                            .read_i32()
                            .expect("Raw reported origin")
                            .cast_unsigned(),
                        expected
                    );
                }
                assert_eq!(
                    reader
                        .read_i32()
                        .expect("Independent prediction timestamp")
                        .cast_unsigned(),
                    PREDICTION_TIME
                );
                assert_eq!(reader.read_bits(3).expect("Logical EOF"), 3);
                assert!(usize::from(plan.user_command_count()) <= MAX_PACKET_USER_COMMANDS);
            }
        }
    }
}
#[test]
fn movement_reserves_its_entire_batch_before_selecting_the_oldest_fitting_reliable_prefix() {
    let input = commands(32);
    let first = vec![b'a'; 900];
    let second = vec![b'b'; 900];
    let plan = build(
        BaselineProtocol::Cod4x21,
        7,
        b"Private.After",
        &input,
        &[&first, &second],
        None,
    );
    assert_eq!((plan.command_count(), plan.user_command_count()), (1, 32));
    assert_eq!(plan.last_command_sequence(), Some(1));
    let decoded = body(&plan, BaselineProtocol::Cod4x21, 7, b"Private.After");
    let mut reader = MessageReader::try_new(&decoded).expect("Body");
    assert_eq!(reader.read_bits(3).expect("Oldest reliable"), 2);
    assert_eq!(reader.read_i32().expect("Sequence"), 1);
    let mut text = [0; 1024];
    let length = reader
        .read_legacy_string(&mut text, MessageStringLimit::Normal)
        .expect("Whole reliable string");
    assert_eq!(&text[..length], first);
    assert_eq!(
        reader
            .read_bits(3)
            .expect("Movement follows without a second reliable"),
        1
    );
    assert_eq!(reader.read_u8().expect("Full batch retained"), 32);
}
