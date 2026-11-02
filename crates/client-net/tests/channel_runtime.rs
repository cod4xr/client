// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent UDP wire tests for owned ordinary-channel integration.
#![allow(
    clippy::expect_used,
    reason = "Bounded loopback tests join independent responders."
)]
mod channel_support;
#[path = "channel_cases/command_history.rs"]
mod command_history;
#[path = "channel_cases/control.rs"]
mod control;
#[path = "channel_cases/limits.rs"]
mod limits;
#[path = "channel_cases/mixed_services/mod.rs"]
mod mixed_services;
#[path = "channel_cases/movement/mod.rs"]
mod movement;
#[path = "channel_cases/pre_game.rs"]
mod pre_game;
#[path = "channel_cases/receive.rs"]
mod receive;
#[path = "channel_cases/reliable_body.rs"]
mod reliable_body;
#[path = "channel_cases/reliable_framing.rs"]
mod reliable_framing;
#[path = "channel_cases/reliable_receive.rs"]
mod reliable_receive;
#[path = "channel_cases/reliable_send/mod.rs"]
mod reliable_send;
#[path = "channel_cases/send.rs"]
mod send;
#[path = "channel_cases/server_payload.rs"]
mod server_payload;
#[path = "channel_cases/services.rs"]
mod services;
#[path = "channel_cases/snapshot_messages/mod.rs"]
mod snapshot_messages;
mod support;
