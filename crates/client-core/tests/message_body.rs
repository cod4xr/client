// SPDX-License-Identifier: AGPL-3.0-or-later
//! Whole-body budgets and shared-codec view rebasing with independent native vectors.
#![allow(
    clippy::expect_used,
    reason = "Finite field traces assert checked fixtures."
)]
#[path = "message_body_cases/indices.rs"]
mod indices;
#[path = "message_body_cases/limits.rs"]
mod limits;
#[path = "message_body_cases/views.rs"]
mod views;

use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4xr_client_core::message_body::BodyReader;

fn state(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
fn padded(payload: &[u8], padding: usize) -> Vec<u8> {
    let mut input = vec![0xfe; padding];
    input.extend_from_slice(payload);
    input
}
fn skip_padding(reader: &mut BodyReader<'_>, length: usize) {
    let mut output = vec![0; length];
    reader
        .read_bytes(&mut output)
        .expect("Bounded delegated byte spans");
    assert!(output.iter().all(|&byte| byte == 0xfe));
}
