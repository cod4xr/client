// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent framing vectors, arbitrary boundaries and window-to-message streaming.
#![allow(
    clippy::expect_used,
    reason = "Finite fixtures use explicit checked capacities."
)]

#[path = "framing_cases/codec.rs"]
mod codec;
#[path = "framing_cases/stream.rs"]
mod stream;
#[path = "framing_cases/window.rs"]
mod window;

const NATIVE_SHAPED: [u8; 13] = [9, 0, 0, 0, 1, 0, 0, 0, 1, 0xfe, 0, 0x92, 0xff];
