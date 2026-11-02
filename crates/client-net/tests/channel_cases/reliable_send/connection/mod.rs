// SPDX-License-Identifier: AGPL-3.0-or-later
mod failures;
mod gamestate;
mod gamestate_preparation;
mod gamestate_preparation_errors;
mod lifecycle;
mod limits;
mod receipt;

use crate::channel_support::acknowledged_for;
use cod4xr_client_net::{ConnectionSend, NetworkError, ReliableConnection, SentReliableDatagram};
use cod4xr_reliable::{
    endpoint::{EndpointConfig, EndpointStorage, ReliableEndpoint},
    receive::ReceiveSlot,
    send::SendSlot,
};
use std::{net::SocketAddr, time::Instant};

#[derive(Clone, Eq, PartialEq)]
struct Buffers {
    ordinary: Vec<u8>,
    send: Vec<u8>,
    send_slots: Vec<SendSlot>,
    incoming: Vec<u8>,
    receive_slots: Vec<ReceiveSlot>,
    body: Vec<u8>,
}
impl Buffers {
    fn new() -> Self {
        Self {
            ordinary: vec![0xa5; 1300],
            send: vec![0xa5; 4800],
            send_slots: vec![SendSlot::default(); 4],
            incoming: vec![0xa5; 3600],
            receive_slots: vec![ReceiveSlot::default(); 3],
            body: vec![0xa5; 32],
        }
    }
    fn storage(&mut self) -> (&mut [u8], EndpointStorage<'_>) {
        (
            &mut self.ordinary,
            EndpointStorage {
                send_bytes: &mut self.send,
                send_slots: &mut self.send_slots,
                receive_bytes: &mut self.incoming,
                receive_slots: &mut self.receive_slots,
                body: &mut self.body,
            },
        )
    }
    fn connect(
        &mut self,
        peer: SocketAddr,
        protocol: i32,
        config: EndpointConfig,
    ) -> Result<ReliableConnection<'_>, NetworkError> {
        let acknowledged = acknowledged_for(peer, protocol);
        let (ordinary, storage) = self.storage();
        acknowledged.into_reliable(ordinary, storage, config)
    }
    fn prime(&mut self) {
        let (_, storage) = self.storage();
        let mut previous = ReliableEndpoint::new(
            std::time::Duration::ZERO,
            EndpointConfig::default(),
            storage,
        )
        .expect("Previous endpoint");
        let _ = previous
            .enqueue(b"PrivatePrevious")
            .expect("Retained queue metadata");
    }
}

fn send_when_due(
    connection: &mut ReliableConnection<'_>,
    deadline: Instant,
) -> SentReliableDatagram {
    let mut outcome = None;
    for _ in 0..8 {
        match connection.send_due(deadline).expect("Paced send") {
            ConnectionSend::Sent(sent) => {
                outcome = Some(sent);
                break;
            }
            ConnectionSend::WaitUntil(at) => {
                assert!(
                    connection
                        .receive_until(at, |_| ())
                        .expect("Wait")
                        .is_none()
                );
            }
        }
    }
    outcome.expect("One bounded due datagram")
}
