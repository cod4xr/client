// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit control profiles, golden wire bytes and bounded prefix construction.
#![allow(
    clippy::expect_used,
    reason = "Finite source-grounded wire and storage fixtures."
)]
#[path = "control_cases/limits.rs"]
mod limits;
#[path = "control_cases/movement/mod.rs"]
mod movement;
#[path = "control_cases/wire.rs"]
mod wire;

use cod4x_protocol::{compression, obfuscation};
use cod4xr_client_core::{
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    control::{ClientControlHeader, ControlMessagePlan, ServerCommandKey},
};

const CHALLENGE: i32 = 0x1234_5678;

fn legacy() -> ClientControlHeader<'static> {
    ClientControlHeader::Legacy6 {
        server_id: 0x12,
        message_acknowledge: 0x0102_0304,
        server_command: ServerCommandKey::initial(),
    }
}

fn modern() -> ClientControlHeader<'static> {
    ClientControlHeader::Cod4x21 {
        server_id: 0x1122_3344,
        message_acknowledge: 5,
        server_command: ServerCommandKey::initial(),
        config_acknowledge: 7,
    }
}

fn build(header: ClientControlHeader<'_>, commands: &[&[u8]]) -> ControlMessagePlan {
    let mut storage = vec![0xa5; (commands.len().max(1) + 1) * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut storage).expect("History");
    for bytes in commands {
        history.enqueue(bytes).expect("Queued");
    }
    ControlMessagePlan::new(header, CHALLENGE, &history).expect("Owned control frame")
}

fn decoded_body(frame: &ControlMessagePlan) -> Vec<u8> {
    let mut bytes = frame.as_bytes()[16..].to_vec();
    obfuscation::apply_client_payload(&mut bytes, 0x1234_5678, 0x1122_3344, 5, b"")
        .expect("Undo compatibility transform for structural boundary checks");
    let mut decoded = [0; 4096];
    let length = compression::decompress(&bytes, &mut decoded).expect("Bounded payload");
    decoded[..length].to_vec()
}
