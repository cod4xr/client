// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::wire;
use super::Buffers;
use crate::{channel_support::accept, support::server};
use cod4xr_client_net::{ConnectionReceive, NetworkError};
use cod4xr_reliable::{
    endpoint::{EndpointConfig, EndpointError},
    framing::FrameError,
};
use std::{
    thread,
    time::{Duration, Instant},
};

#[test]
fn terminal_framing_failure_retains_work_counts_and_prevents_further_socket_operations() {
    let socket = server();
    let address = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        socket
            .send_to(&wire(0, 0, &[], 4, &(-1_i32).to_le_bytes()), client)
            .expect("Invalid frame prefix");
        socket
            .send_to(&wire(1, 0, &[], 4, b"\x01\0\0\0x"), client)
            .expect("Further queued packet");
    });
    let mut buffers = Buffers::new();
    let mut connection = buffers
        .connect(address, 21, EndpointConfig::default())
        .expect("Owner");
    let deadline = Instant::now() + Duration::from_secs(4);
    assert!(matches!(
        connection
            .receive_until(deadline, |_| ())
            .expect("Receive")
            .expect("Packet"),
        ConnectionReceive::Accepted(_)
    ));
    let drain = connection.drain(&mut [0; 8], 32, 8);
    assert_eq!(
        (drain.usage.bytes, drain.outcome),
        (4, Err(FrameError::Length))
    );
    let before = connection.stats();
    assert!(before.endpoint.failed);
    assert_eq!(
        (
            before.reliable.framing_failures,
            before.reliable.drained_bytes
        ),
        (1, 4)
    );
    assert!(matches!(
        connection.receive_until(deadline, |_| ()),
        Err(NetworkError::Endpoint(EndpointError::Failed))
    ));
    assert!(matches!(
        connection.send_due(deadline),
        Err(NetworkError::Endpoint(EndpointError::Failed))
    ));
    assert!(matches!(
        connection.ordinary_sender(),
        Err(NetworkError::Endpoint(EndpointError::Failed))
    ));
    assert!(matches!(
        connection.enqueue(b"new"),
        Err(NetworkError::Endpoint(EndpointError::Failed))
    ));
    assert_eq!(
        connection.drain(&mut [0; 8], 32, 8).outcome,
        Err(FrameError::Failed)
    );
    assert!(connection.frame().is_none());
    assert!(!connection.consume_frame());
    assert_eq!(connection.stats(), before);
    assert_eq!(connection.stats().receive.datagrams, 1);
    responder.join().expect("Peer");
}
