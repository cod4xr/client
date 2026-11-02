// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    net::UdpSocket,
    thread,
    time::{Duration, Instant},
};

use super::{storage, wire};
use crate::{
    channel_support::{accept, acknowledged},
    support::{receive, server},
};
use cod4xr_client_net::{ChannelEvent, ClientReceiver};
use cod4xr_reliable::{
    endpoint::{
        DrainStatus, EndpointConfig, EndpointError, EndpointStorage, IngestReport, ReliableEndpoint,
    },
    receive::ReceiveSlot,
    schedule::SchedulePoll,
    send::SendError,
};

fn peer(socket: UdpSocket) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let client = accept(&socket);
        let (bytes, source) = receive(&socket);
        assert_eq!(source, client);
        assert_eq!(bytes, wire(0, 0, &[], 4, b"\x06\0\0\0client"));
        for acknowledgement in [2, 1] {
            let mut bytes = wire(0, acknowledgement, &[], 4, b"\x06\0\0\0server");
            bytes[14] = 1; // Force-ACK may apply only after the entire packet is admitted.
            socket.send_to(&bytes, client).expect("Combined data/ACK");
        }
        let (bytes, source) = receive(&socket);
        assert_eq!(source, client);
        assert_eq!(bytes, wire(-1, 1, &[], 4, b""));
    })
}

fn receive_endpoint(
    receiver: &mut ClientReceiver<'_, '_>,
    endpoint: &mut ReliableEndpoint<'_>,
    tick: u64,
    deadline: Instant,
) -> Result<IngestReport, EndpointError> {
    receiver
        .receive_until(deadline, |event| {
            let ChannelEvent::Auxiliary(message) = event else {
                return None;
            };
            Some(endpoint.ingest(
                message.reliable_packet().expect("Complete reliable body"),
                Duration::from_millis(tick),
            ))
        })
        .expect("Receive")
        .expect("One datagram")
        .expect("Auxiliary family")
}

#[test]
fn unified_endpoint_validates_whole_udp_packets_before_force_ack_frame_delivery_and_queue_release()
{
    let socket = server();
    let address = socket.local_addr().expect("Peer");
    let responder = peer(socket);
    let mut ordinary_storage = [0; 1300];
    let mut channel = acknowledged(address)
        .into_channel(&mut ordinary_storage)
        .expect("Channel");
    let (mut send_bytes, mut send_slots) = storage(4);
    let mut receive_bytes = [0; 3600];
    let mut receive_slots = [ReceiveSlot::default(); 3];
    let mut body = [0xa5; 32];
    let mut endpoint = ReliableEndpoint::new(
        Duration::ZERO,
        EndpointConfig::default(),
        EndpointStorage {
            send_bytes: &mut send_bytes,
            send_slots: &mut send_slots,
            receive_bytes: &mut receive_bytes,
            receive_slots: &mut receive_slots,
            body: &mut body,
        },
    )
    .expect("Fresh endpoint");
    let _ = endpoint.enqueue(b"client").expect("Atomic framed queue");
    let deadline = Instant::now() + Duration::from_secs(4);
    {
        let mut parts = channel.split();
        let plan = match endpoint.poll(Duration::ZERO).expect("Schedule") {
            SchedulePoll::Send(plan) => Some(plan),
            SchedulePoll::WaitUntil(_) => None,
        }
        .expect("Data eligible");
        let _ = parts
            .sender
            .send_scheduled_reliable(plan, deadline)
            .expect("Actual frame send");
        assert_eq!(
            receive_endpoint(&mut parts.receiver, &mut endpoint, 25, deadline),
            Err(EndpointError::Send(SendError::AckFuture))
        );
        assert_eq!(
            (
                endpoint.state().outgoing_fragments,
                endpoint.state().incoming_bytes
            ),
            (1, 0)
        );
        assert!(
            matches!(endpoint.poll(Duration::from_millis(50)).expect("Invalid force did not apply"), SchedulePoll::WaitUntil(at) if at == Duration::from_millis(250))
        );
        let accepted = receive_endpoint(&mut parts.receiver, &mut endpoint, 50, deadline)
            .expect("Whole admission");
        assert!(accepted.force_ack);
        assert_eq!(accepted.acknowledgement.released_fragments, 1);
        let drain = endpoint.drain(&mut [0; 8], 64, 8);
        assert_eq!(
            (drain.usage.bytes, drain.outcome),
            (10, Ok(DrainStatus::Ready))
        );
        assert_eq!(endpoint.frame().expect("Complete only").body(), b"server");
        let plan = match endpoint
            .poll(Duration::from_millis(50))
            .expect("Validated force")
        {
            SchedulePoll::Send(plan) => Some(plan),
            SchedulePoll::WaitUntil(_) => None,
        }
        .expect("ACK eligible");
        let sent = parts
            .sender
            .send_scheduled_reliable(plan, deadline)
            .expect("Consumed ACK");
        assert_eq!((sent.fragment, sent.bytes), (None, 20));
        assert_eq!(parts.sender.next_sequence(), 1);
    }
    responder.join().expect("Responder");
    assert_eq!(
        (
            endpoint.state().outgoing_fragments,
            endpoint.state().incoming_fragments,
            endpoint.state().incoming_acknowledge
        ),
        (0, 0, 1)
    );
    assert!(endpoint.state().frame_ready);
    let usage = channel.send_usage();
    assert_eq!(
        (
            usage.messages_reserved,
            usage.datagrams_sent,
            usage.reliable_datagrams_sent,
            usage.bytes_sent
        ),
        (0, 2, 2, 50)
    );
    assert_eq!(channel.receive_usage().auxiliary, 2);
}
