// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::wire;
use super::Buffers;
use crate::{channel_support::accept, support::server};
use cod4xr_client_net::{ConnectionReceive, ReliableRejection};
use cod4xr_reliable::{ReliableError, endpoint::EndpointConfig};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

#[test]
fn receipt_clock_is_observed_after_blocking_io_and_rejected_or_expired_receives_do_not_refresh_it()
{
    let socket = server();
    let address = socket.local_addr().expect("Peer");
    let (release, completion) = mpsc::sync_channel(0);
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        let mut invalid = wire(-1, 0, &[], 4, b"");
        invalid[18..20].copy_from_slice(&1_i16.to_le_bytes());
        socket.send_to(&invalid, client).expect("Invalid suffix");
        completion
            .recv_timeout(Duration::from_secs(5))
            .expect("Begin delayed reply");
        thread::sleep(Duration::from_millis(25));
        socket
            .send_to(&wire(-1, 0, &[], 4, b""), client)
            .expect("Valid ACK");
    });
    let mut buffers = Buffers::new();
    let mut connection = buffers
        .connect(address, 21, EndpointConfig::default())
        .expect("Owner");
    let deadline = Instant::now() + Duration::from_secs(4);
    let mut other_called = false;
    let rejected = connection
        .receive_until(deadline, |_| {
            other_called = true;
        })
        .expect("Receive")
        .expect("Packet");
    assert!(matches!(
        rejected,
        ConnectionReceive::Rejected(ReliableRejection::Body(ReliableError::LengthMismatch))
    ));
    assert!(connection.stats().reliable.last_accepted_at.is_none());
    release.send(()).expect("Release delayed peer");
    assert!(matches!(
        connection
            .receive_until(deadline, |_| {
                other_called = true;
            })
            .expect("Receive")
            .expect("Packet"),
        ConnectionReceive::Accepted(_)
    ));
    assert!(
        connection
            .stats()
            .reliable
            .last_accepted_at
            .expect("Actual receipt")
            >= Duration::from_millis(25)
    );
    let before = connection.stats();
    assert!(
        connection
            .receive_until(Instant::now(), |_| {
                other_called = true;
            })
            .expect("Expired wait")
            .is_none()
    );
    assert_eq!(connection.stats(), before);
    assert!(!other_called);
    responder.join().expect("Peer");
}
