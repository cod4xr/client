// SPDX-License-Identifier: AGPL-3.0-or-later
//! Whole packet admission, frame boundaries and bounded end-to-end transport state.
#![allow(
    clippy::expect_used,
    reason = "Explicit finite endpoint fixtures assert checked setup."
)]
#[path = "endpoint_cases/construction.rs"]
mod construction;
#[path = "endpoint_cases/drain.rs"]
mod drain;
#[path = "endpoint_cases/ingest.rs"]
mod ingest;
#[path = "endpoint_cases/model.rs"]
mod model;

use cod4xr_reliable::{
    ReliableHeader, ReliablePacket, SelectiveAcks,
    endpoint::{EndpointConfig, EndpointError, EndpointStorage, ReliableEndpoint},
    receive::{ReceiveSlot, required_receive_bytes},
    schedule::SchedulePoll,
    send::{SendSlot, required_send_bytes},
};
use std::time::Duration;

struct Buffers {
    send: Vec<u8>,
    send_slots: Vec<SendSlot>,
    receive: Vec<u8>,
    receive_slots: Vec<ReceiveSlot>,
    body: Vec<u8>,
}
impl Buffers {
    fn new(send: usize, receive: usize, body: usize) -> Self {
        Self {
            send: vec![0xa5; required_send_bytes(send).expect("Send budget")],
            send_slots: vec![SendSlot::default(); send],
            receive: vec![0xa5; required_receive_bytes(receive).expect("Receive budget")],
            receive_slots: vec![ReceiveSlot::default(); receive],
            body: vec![0xa5; body],
        }
    }
    fn storage(&mut self) -> EndpointStorage<'_> {
        EndpointStorage {
            send_bytes: &mut self.send,
            send_slots: &mut self.send_slots,
            receive_bytes: &mut self.receive,
            receive_slots: &mut self.receive_slots,
            body: &mut self.body,
        }
    }
    fn endpoint(&mut self) -> ReliableEndpoint<'_> {
        ReliableEndpoint::new(ms(0), EndpointConfig::default(), self.storage())
            .expect("Fresh endpoint")
    }
}
fn ms(value: u64) -> Duration {
    Duration::from_millis(value)
}
fn packet(
    sequence: Option<u32>,
    acknowledge: u32,
    force: bool,
    payload: &[u8],
) -> ReliablePacket<'_> {
    ReliablePacket::try_new(
        ReliableHeader {
            sequence,
            acknowledge,
            flags: u8::from(force),
            selective: SelectiveAcks::default(),
            sender_window: i16::MAX,
        },
        payload,
    )
    .expect("Complete body fixture")
}
fn transmit(
    endpoint: &mut ReliableEndpoint<'_>,
    now: Duration,
) -> Result<Option<Vec<u8>>, EndpointError> {
    match endpoint.poll(now)? {
        SchedulePoll::WaitUntil(_) => Ok(None),
        SchedulePoll::Send(plan) => {
            let mut bytes = vec![0; 1226];
            let sent = plan
                .send_with::<()>(|packet| {
                    let length = packet.encode(&mut bytes).expect("Complete body");
                    Ok(length)
                })
                .expect("Model OS admission");
            bytes.truncate(sent.bytes);
            Ok(Some(bytes))
        }
    }
}
