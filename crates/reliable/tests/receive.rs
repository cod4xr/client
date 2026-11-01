// SPDX-License-Identifier: AGPL-3.0-or-later
//! Model streams, bounded empty-fragment work and transactional receive admission.
#![allow(clippy::expect_used, reason = "Finite fixtures assert checked setup.")]

#[path = "receive_cases/admission.rs"]
mod admission;
#[path = "receive_cases/selective.rs"]
mod selective;
#[path = "receive_cases/streams.rs"]
mod streams;

use cod4xr_reliable::{ReliableHeader, ReliablePacket, SelectiveAcks, receive::ReceiveSlot};

fn packet(sequence: Option<u32>, payload: &[u8]) -> ReliablePacket<'_> {
    ReliablePacket::try_new(
        ReliableHeader {
            sequence,
            acknowledge: 0,
            flags: 0,
            selective: SelectiveAcks::default(),
            sender_window: 4,
        },
        payload,
    )
    .expect("Representable body")
}

fn storage(capacity: usize) -> (Vec<u8>, Vec<ReceiveSlot>) {
    (
        vec![0xa5; cod4xr_reliable::receive::required_receive_bytes(capacity).expect("Span")],
        vec![ReceiveSlot::default(); capacity],
    )
}
