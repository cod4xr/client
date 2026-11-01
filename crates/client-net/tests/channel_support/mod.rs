// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    net::{SocketAddr, UdpSocket},
    time::Instant,
};

use cod4xr_client_net::{
    AcknowledgedSocket, AuxiliaryKind, ChannelDrop, ChannelEvent, ClientReceiver, ConnectOutcome,
};

use crate::support;

pub fn accept(socket: &UdpSocket) -> SocketAddr {
    let client = support::reply_challenge(socket);
    let (request, source) = support::receive(socket);
    assert_eq!(source, client);
    assert!(request.starts_with(b"\xff\xff\xff\xffconnect \""));
    socket
        .send_to(&support::packet(b"connectResponse mods/PrivateMod"), client)
        .expect("Acknowledge");
    client
}

pub fn acknowledged(peer: SocketAddr) -> AcknowledgedSocket {
    match support::challenged(peer)
        .into_pending(support::settings(), support::budget())
        .expect("Pending")
        .run()
        .outcome
        .expect("Outcome")
    {
        ConnectOutcome::Acknowledged(socket) => Some(socket),
        ConnectOutcome::Refused(_) => None,
    }
    .expect("Acknowledged socket")
}

pub fn ordinary(sequence: u32, body: &[u8]) -> Vec<u8> {
    let mut bytes = sequence.to_le_bytes().to_vec();
    bytes.extend_from_slice(body);
    bytes
}

pub fn fragment(sequence: u32, start: u32, body: &[u8]) -> Vec<u8> {
    let mut bytes = (sequence | 0x8000_0000).to_le_bytes().to_vec();
    bytes.extend_from_slice(&start.to_le_bytes());
    bytes.extend_from_slice(
        &i16::try_from(body.len())
            .expect("Finite fragment")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(body);
    bytes
}

#[derive(Debug, Eq, PartialEq)]
pub enum Seen {
    Message(u32, u32, Vec<u8>),
    Pending,
    Dropped(ChannelDrop),
    Auxiliary(AuxiliaryKind, Vec<u8>),
}

pub fn read(receiver: &mut ClientReceiver<'_, '_>, deadline: Instant) -> Seen {
    receiver
        .receive_until(deadline, |event| {
            let debug = format!("{event:?}");
            for private in [
                "PrivateBody",
                "PrivateNotice",
                "PrivateReliable",
                "PrivateUpdate",
            ] {
                assert!(!debug.contains(private));
            }
            match event {
                ChannelEvent::Message(message) => Seen::Message(
                    message.sequence(),
                    message.skipped_sequences(),
                    message.body().to_vec(),
                ),
                ChannelEvent::FragmentPending => Seen::Pending,
                ChannelEvent::Dropped(reason) => Seen::Dropped(reason),
                ChannelEvent::Auxiliary(message) => {
                    Seen::Auxiliary(message.kind(), message.body().to_vec())
                }
            }
        })
        .expect("Receive operation")
        .expect("One packet before deadline")
}
