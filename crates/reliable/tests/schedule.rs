// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact-clock pacing and combined fragment/timer admission without wall-clock sleeps.
#![allow(
    clippy::expect_used,
    reason = "Explicit-time fixtures assert checked setup and phases."
)]
#[path = "schedule_cases/acknowledgements.rs"]
mod acknowledgements;
#[path = "schedule_cases/force.rs"]
mod force;
#[path = "schedule_cases/timing.rs"]
mod timing;
#[path = "schedule_cases/transactions.rs"]
mod transactions;

use std::time::Duration;

use cod4xr_reliable::{
    ReliableHeader, ReliablePacket, SelectiveAckRange, SelectiveAcks,
    receive::{ReceiveSlot, ReceiveWindow, required_receive_bytes},
    schedule::{ReliableScheduler, SchedulePoll, ScheduledDatagram, ScheduledSend},
    send::{SendSlot, SendWindow, required_send_bytes},
};

fn ms(value: u64) -> Duration {
    Duration::from_millis(value)
}
fn send_storage(capacity: usize) -> (Vec<u8>, Vec<SendSlot>) {
    (
        vec![0; required_send_bytes(capacity).expect("Send geometry")],
        vec![SendSlot::default(); capacity],
    )
}
fn receive_storage(capacity: usize) -> (Vec<u8>, Vec<ReceiveSlot>) {
    (
        vec![0; required_receive_bytes(capacity).expect("Receive geometry")],
        vec![ReceiveSlot::default(); capacity],
    )
}
fn ready<'scheduler, 'window, 'buffers>(
    scheduler: &'scheduler mut ReliableScheduler,
    now: Duration,
    outgoing: &'window mut SendWindow<'buffers>,
    incoming: &ReceiveWindow<'_>,
) -> ScheduledDatagram<'scheduler, 'window, 'buffers> {
    match scheduler.poll(now, outgoing, incoming).expect("Poll") {
        SchedulePoll::Send(plan) => Some(plan),
        SchedulePoll::WaitUntil(_) => None,
    }
    .expect("Eligible datagram")
}
fn wait(
    scheduler: &mut ReliableScheduler,
    now: Duration,
    outgoing: &mut SendWindow<'_>,
    incoming: &ReceiveWindow<'_>,
) -> Duration {
    match scheduler.poll(now, outgoing, incoming).expect("Poll") {
        SchedulePoll::WaitUntil(at) => Some(at),
        SchedulePoll::Send(_) => None,
    }
    .expect("Pacing wait")
}
fn complete(plan: ScheduledDatagram<'_, '_, '_>) -> ScheduledSend {
    plan.send_with::<()>(|packet| {
        assert_eq!(packet.header().flags, 0);
        assert!(packet.payload().len() <= 1200);
        Ok(packet.encoded_len() + 6)
    })
    .expect("Explicit model I/O admission")
}
fn ack(cumulative: u32, ranges: &[(u16, u16)]) -> ReliablePacket<'static> {
    let ranges: Vec<_> = ranges
        .iter()
        .map(|&(offset, length)| SelectiveAckRange { offset, length })
        .collect();
    ReliablePacket::try_new(
        ReliableHeader {
            sequence: None,
            acknowledge: cumulative,
            flags: 0,
            selective: SelectiveAcks::new(&ranges).expect("Ranges"),
            sender_window: 4,
        },
        b"",
    )
    .expect("ACK-only")
}
fn receive_slot(incoming: &mut ReceiveWindow<'_>, sequence: u32) {
    let packet = ReliablePacket::try_new(
        ReliableHeader {
            sequence: Some(sequence),
            ..ack(0, &[]).header()
        },
        b"",
    )
    .expect("Empty data fixture");
    let _ = incoming
        .prepare(packet)
        .expect("Incoming layout fixture")
        .commit();
}
