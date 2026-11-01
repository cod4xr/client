// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent UDP wire tests for owned ordinary-channel integration.
#![allow(
    clippy::expect_used,
    reason = "Bounded loopback tests join independent responders."
)]
mod channel_support;
#[path = "channel_cases/control.rs"]
mod control;
#[path = "channel_cases/limits.rs"]
mod limits;
#[path = "channel_cases/receive.rs"]
mod receive;
#[path = "channel_cases/send.rs"]
mod send;
mod support;
