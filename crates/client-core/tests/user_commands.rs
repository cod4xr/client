// SPDX-License-Identifier: AGPL-3.0-or-later
//! Outgoing native full commands checked against physical vectors/shared decoding.
#![allow(
    clippy::expect_used,
    reason = "Bounded command fixtures and exact assertions."
)]
#[path = "user_command_cases/history/mod.rs"]
mod history;
#[path = "user_command_cases/limits.rs"]
mod limits;
#[path = "user_command_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::{
    MessageReader,
    user_command::{self, LegacyUserCommand},
};
use cod4xr_client_core::user_commands::{
    AbsoluteUserCommandPlan, MAX_ABSOLUTE_COMMAND_BYTES, MovementAxis, UserCommandEncodeError,
    UserCommandInput, UserCommandMelee,
};

fn zero() -> UserCommandInput {
    UserCommandInput {
        server_time: 0,
        buttons: 0,
        angles: [0; 3],
        weapon: 0,
        offhand: 0,
        movement: [MovementAxis::Stationary; 2],
        selected_location: None,
        melee: None,
    }
}
fn baseline() -> LegacyUserCommand {
    LegacyUserCommand {
        server_time: -123,
        buttons: 0x1f_fffa,
        angles: [0xfedc, 0xba98, 0x7654],
        weapon: 99,
        offhand: 98,
        forward_move: -128,
        right_move: 11,
        melee_charge_yaw: 17.25,
        melee_charge_distance: 197,
        selected_location: [99, 88],
    }
}
fn decode(key: u32, input: UserCommandInput, prior: &LegacyUserCommand) -> LegacyUserCommand {
    let plan = AbsoluteUserCommandPlan::new(key, input).expect("Checked command");
    let mut bytes = [0xa5; MAX_ABSOLUTE_COMMAND_BYTES];
    let length = plan.encode(&mut bytes).expect("Whole command");
    assert_eq!(length, plan.encoded_len());
    let mut reader = MessageReader::try_new(&bytes[..length]).expect("Encoded bytes");
    let command =
        user_command::read_delta(&mut reader, key, prior).expect("Published shared decoder");
    assert_eq!(reader.remaining_bytes(), 0);
    command
}
