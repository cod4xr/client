// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent auxiliary body vectors and transactional capacity/domain boundaries.
#![allow(
    clippy::expect_used,
    reason = "Finite native-shaped body fixtures have explicit fields."
)]
#[path = "body_cases/limits.rs"]
mod limits;
#[path = "body_cases/wire.rs"]
mod wire;

use cod4xr_reliable::{ReliableHeader, SelectiveAcks};

const ACK_ONLY: [u8; 14] = [0xff, 0xff, 0xff, 0xff, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0];
const DATA: [u8; 25] = [
    2, 0, 0, 0, 4, 0, 0, 0, 1, 2, 1, 0, 2, 0, 5, 0, 1, 0, 4, 0, 3, 0, 0xaa, 0, 0xff,
];

fn header() -> ReliableHeader {
    ReliableHeader {
        sequence: Some(0),
        acknowledge: 0,
        flags: 0,
        selective: SelectiveAcks::default(),
        sender_window: 4,
    }
}
