// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    net::UdpSocket,
    thread,
    time::{Duration, Instant},
};

use cod4xr_client_net::NetworkError;
use cod4xr_reliable::{
    receive::{ReceiveSlot, ReceiveWindow},
    schedule::{ReliableScheduler, ScheduleConfig, SchedulePoll, ScheduledDatagram},
    send::SendWindow,
};

use super::{receive_auxiliary, storage, wire};
use crate::{
    channel_support::{accept, acknowledged},
    support::{receive, server},
};

fn ready<'scheduler, 'window, 'buffers>(
    scheduler: &'scheduler mut ReliableScheduler,
    tick: u64,
    outgoing: &'window mut SendWindow<'buffers>,
    incoming: &ReceiveWindow<'_>,
) -> ScheduledDatagram<'scheduler, 'window, 'buffers> {
    match scheduler
        .poll(Duration::from_millis(tick), outgoing, incoming)
        .expect("Poll")
    {
        SchedulePoll::Send(plan) => Some(plan),
        SchedulePoll::WaitUntil(_) => None,
    }
    .expect("Scheduled body")
}

fn assert_wait(
    scheduler: &mut ReliableScheduler,
    tick: u64,
    outgoing: &mut SendWindow<'_>,
    incoming: &ReceiveWindow<'_>,
    until: u64,
) {
    assert!(
        matches!(scheduler.poll(Duration::from_millis(tick), outgoing, incoming).expect("Poll"),
        SchedulePoll::WaitUntil(at) if at == Duration::from_millis(until))
    );
}

fn peer(socket: UdpSocket) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let client = accept(&socket);
        for _ in 0..2 {
            let (bytes, source) = receive(&socket);
            assert_eq!(source, client);
            assert_eq!(bytes, wire(0, 0, &[], 4, b"PrivateRPC"));
        }
        socket
            .send_to(&wire(-1, 1, &[], 4, b""), client)
            .expect("Consumed");
        let (bytes, source) = receive(&socket);
        assert_eq!(source, client);
        assert_eq!(bytes, wire(-1, 0, &[], 4, b""));
    })
}

#[test]
fn scheduled_udp_failure_cancels_both_admissions_and_success_paces_original_retries_and_idle_acks()
{
    let socket = server();
    let address = socket.local_addr().expect("Peer");
    let responder = peer(socket);
    let mut ordinary_storage = [0; 1300];
    let mut channel = acknowledged(address)
        .into_channel(&mut ordinary_storage)
        .expect("Channel");
    let (mut bytes, mut slots) = storage(4);
    let mut outgoing = SendWindow::new(&mut bytes, &mut slots, 4).expect("Outgoing");
    let _ = outgoing.enqueue(b"PrivateRPC").expect("Original");
    let mut receive_bytes = [0; 3600];
    let mut receive_slots = [ReceiveSlot::default(); 3];
    let mut incoming =
        ReceiveWindow::new(&mut receive_bytes, &mut receive_slots).expect("Incoming");
    let mut scheduler =
        ReliableScheduler::new(Duration::ZERO, ScheduleConfig::default()).expect("Scheduler");
    let deadline = Instant::now() + Duration::from_secs(4);
    {
        let mut parts = channel.split();
        let plan = ready(&mut scheduler, 0, &mut outgoing, &incoming);
        assert!(matches!(
            parts.sender.send_scheduled_reliable(plan, Instant::now()),
            Err(NetworkError::Timeout)
        ));
        let plan = ready(&mut scheduler, 0, &mut outgoing, &incoming);
        let sent = parts
            .sender
            .send_scheduled_reliable(plan, deadline)
            .expect("First actual send");
        assert!(sent.fragment.expect("Data").first_transmission);
        assert_wait(&mut scheduler, 0, &mut outgoing, &incoming, 250);
        let plan = ready(&mut scheduler, 250, &mut outgoing, &incoming);
        let sent = parts
            .sender
            .send_scheduled_reliable(plan, deadline)
            .expect("Scheduled retry");
        assert!(
            !sent
                .fragment
                .expect("Same original data")
                .first_transmission
        );
        let _ = receive_auxiliary(&mut parts.receiver, deadline, &mut outgoing, &mut incoming);
        scheduler
            .observe_validated_packet(Duration::from_millis(250), false)
            .expect("Whole packet admitted");
        assert_wait(&mut scheduler, 599, &mut outgoing, &incoming, 600);
        let plan = ready(&mut scheduler, 600, &mut outgoing, &incoming);
        let sent = parts
            .sender
            .send_scheduled_reliable(plan, deadline)
            .expect("Periodic ACK");
        assert_eq!((sent.fragment, sent.bytes), (None, 20));
        assert_wait(&mut scheduler, 600, &mut outgoing, &incoming, 950);
        assert_eq!(parts.sender.next_sequence(), 1);
    }
    responder.join().expect("Responder");
    assert_eq!(
        (
            outgoing.acknowledge(),
            outgoing.queued_fragments(),
            outgoing.buffered_bytes()
        ),
        (1, 0, 0)
    );
    let usage = channel.send_usage();
    assert_eq!(
        (
            usage.messages_reserved,
            usage.datagrams_sent,
            usage.reliable_datagrams_sent,
            usage.bytes_sent,
            usage.failed_datagrams,
            usage.reliable_failed_datagrams
        ),
        (0, 3, 3, 80, 1, 1)
    );
}
