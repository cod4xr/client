// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original-byte ownership, actual-send ACK admission and independent lossy streams.
#![allow(
    clippy::expect_used,
    reason = "Finite fixtures assert checked window setup."
)]

#[path = "send_cases/acks.rs"]
mod acks;
#[path = "send_cases/limits.rs"]
mod limits;
#[path = "send_cases/model.rs"]
mod model;
#[path = "send_cases/queue.rs"]
mod queue;
#[path = "send_cases/transactions.rs"]
mod transactions;

use cod4xr_reliable::{
    ReliableHeader, ReliablePacket, SelectiveAckRange, SelectiveAcks,
    send::{SendSlot, SendWindow, required_send_bytes},
};

fn storage(capacity: usize) -> (Vec<u8>, Vec<SendSlot>) {
    (
        vec![0xa5; required_send_bytes(capacity).expect("Queue geometry")],
        vec![SendSlot::default(); capacity],
    )
}

fn packet(
    sequence: Option<u32>,
    acknowledge: u32,
    ranges: &[(u16, u16)],
) -> ReliablePacket<'static> {
    let ranges: Vec<_> = ranges
        .iter()
        .map(|&(offset, length)| SelectiveAckRange { offset, length })
        .collect();
    ReliablePacket::try_new(
        ReliableHeader {
            sequence,
            acknowledge,
            flags: 1,
            selective: SelectiveAcks::new(&ranges).expect("Representable ranges"),
            sender_window: i16::MAX,
        },
        b"",
    )
    .expect("Wire body only")
}

fn enqueue_fragments(window: &mut SendWindow<'_>, count: u8) {
    for value in 0..count {
        assert_eq!(
            window
                .enqueue(&[value])
                .expect("One call/one fragment")
                .fragments,
            1
        );
    }
}

fn sent(window: &mut SendWindow<'_>, sequence: u32) {
    let _ = window
        .prepare_transmission(sequence)
        .expect("Transmit admission")
        .record_sent();
}
