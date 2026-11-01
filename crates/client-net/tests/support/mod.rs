// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::expect_used, reason = "Bounded independent loopback fixtures.")]
use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use cod4x_protocol::{InfoLimit, InfoString};
use cod4xr_client_core::{
    connect::{ConnectBudget, NoAuthConnectSettings},
    handshake::ChallengeBudget,
};
use cod4xr_client_net::{ChallengedSocket, challenge};

pub fn server() -> UdpSocket {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Loopback peer");
    socket
        .set_read_timeout(Some(Duration::from_secs(4)))
        .expect("Finite fixture deadline");
    socket
}

pub fn packet(body: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0xff; 4];
    bytes.extend_from_slice(body);
    bytes
}

pub fn reply_challenge(socket: &UdpSocket) -> SocketAddr {
    let (request, client) = receive(socket);
    assert!(request.starts_with(b"\xff\xff\xff\xffgetchallenge "));
    let nonce: i32 = std::str::from_utf8(&request[17..])
        .expect("ASCII nonce")
        .parse()
        .expect("One signed nonce, no credentials");
    let response = packet(format!("challengeResponse -2147481234 {nonce} 0 xproto 18").as_bytes());
    socket
        .send_to(&response, client)
        .expect("Correlated challenge");
    client
}

pub fn receive(socket: &UdpSocket) -> (Vec<u8>, SocketAddr) {
    let mut incoming = vec![0; 65_535];
    let (length, client) = socket
        .recv_from(&mut incoming)
        .expect("Finite packet receive");
    incoming.truncate(length);
    (incoming, client)
}

pub fn challenged(peer: SocketAddr) -> ChallengedSocket {
    challenge(
        peer,
        ChallengeBudget {
            timeout: Duration::from_secs(2),
            retry_interval: Duration::from_millis(500),
            max_attempts: 2,
            max_datagrams: 8,
        },
    )
    .outcome
    .expect("Selected challenge")
}

pub fn budget() -> ConnectBudget {
    ConnectBudget {
        timeout: Duration::from_secs(2),
        retry_interval: Duration::from_millis(100),
        max_attempts: 2,
        max_datagrams: 8,
    }
}

pub fn settings() -> NoAuthConnectSettings<'static> {
    NoAuthConnectSettings {
        protocol: 21,
        qport: 40_000,
        version: b"21.8",
        userinfo: InfoString::parse(
            b"\\name\\PrivatePlayer\\password\\PrivatePassword",
            InfoLimit::Normal,
        )
        .expect("Caller userinfo"),
    }
}
