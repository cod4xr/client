// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4x_transport::{ChannelError, ReceiveDrop};
use cod4xr_client_net::{ChannelDrop, NetworkError, ReceiveUsage};

use crate::{
    channel_support::{Seen, accept, acknowledged, fragment, ordinary, read},
    support::server,
};

#[test]
fn capacity_failure_preserves_partial_data_and_host_can_discard_without_advancing_sequence() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        for bytes in [
            fragment(1, 0, &[b'x'; 1300]),
            fragment(1, 1300, b"overflow"),
            ordinary(2, b"new"),
        ] {
            socket.send_to(&bytes, client).expect("Bounded reassembly");
        }
    });
    let mut storage = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut storage)
        .expect("Channel");
    let deadline = Instant::now() + Duration::from_secs(2);
    {
        let mut receiver = channel.split().receiver;
        assert_eq!(read(&mut receiver, deadline), Seen::Pending);
        assert_eq!(
            read(&mut receiver, deadline),
            Seen::Dropped(ChannelDrop::State(ReceiveDrop::Capacity))
        );
        assert_eq!(
            (receiver.partial_bytes(), receiver.incoming_sequence()),
            (1300, 0)
        );
        receiver.discard_partial();
        assert_eq!(
            (receiver.partial_bytes(), receiver.incoming_sequence()),
            (0, 0)
        );
        assert_eq!(
            read(&mut receiver, deadline),
            Seen::Message(2, 1, b"new".to_vec())
        );
    }
    responder.join().expect("Responder");
    assert!(storage.iter().all(|&byte| byte == b'x'));
}

#[test]
fn expired_receive_deadline_does_not_invoke_callback_or_charge_work() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let (release, wait) = mpsc::channel();
    let responder = thread::spawn(move || {
        accept(&socket);
        wait.recv_timeout(Duration::from_secs(4))
            .expect("Client checked expiry");
    });
    let mut storage = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut storage)
        .expect("Channel");
    let deadline = Instant::now();
    let mut calls = 0;
    {
        let mut receiver = channel.split().receiver;
        for _ in 0..2 {
            assert_eq!(
                receiver
                    .receive_until(deadline, |_| {
                        calls += 1;
                    })
                    .expect("Expired wait"),
                None
            );
        }
        assert_eq!(receiver.usage(), ReceiveUsage::default());
    }
    assert_eq!(calls, 0);
    release.send(()).expect("Release peer");
    responder.join().expect("Responder");
}

#[test]
fn storage_above_profile_limit_rejects_without_constructing_a_channel() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || {
        accept(&socket);
    });
    let acknowledged = acknowledged(peer);
    responder.join().expect("Responder");
    let mut storage = vec![0; MAX_MESSAGE_BYTES + 1];
    assert!(matches!(
        acknowledged.into_channel(&mut storage),
        Err(NetworkError::Channel(ChannelError::MessageTooLarge))
    ));
}
