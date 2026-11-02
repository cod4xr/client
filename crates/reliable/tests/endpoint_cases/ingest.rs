// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{Buffers, ms, packet, transmit};
use cod4xr_reliable::{
    endpoint::EndpointError,
    receive::ReceiveError,
    schedule::{ScheduleError, SchedulePoll},
    send::SendError,
};

#[test]
fn rejected_force_ack_and_clock_observations_cannot_change_other_state_owners() {
    let mut buffers = Buffers::new(4, 3, 32);
    let mut endpoint = buffers.endpoint();
    let before = endpoint.state();
    assert_eq!(
        endpoint.ingest(packet(Some(0), 1, true, b"x"), ms(100)),
        Err(EndpointError::Send(SendError::AckFuture))
    );
    assert_eq!(endpoint.state(), before);
    assert!(
        matches!(endpoint.poll(ms(0)).expect("Clock was not advanced"), SchedulePoll::WaitUntil(at) if at == ms(350))
    );
    assert!(matches!(
        endpoint.poll(ms(100)).expect("Observe wait"),
        SchedulePoll::WaitUntil(_)
    ));
    assert_eq!(
        endpoint.ingest(packet(Some(0), 0, true, b"x"), ms(99)),
        Err(EndpointError::Schedule(ScheduleError::ClockRegressed))
    );
    assert_eq!(endpoint.state(), before);
    assert!(
        matches!(endpoint.poll(ms(100)).expect("Force was not applied"), SchedulePoll::WaitUntil(at) if at == ms(350))
    );
}

#[test]
fn unsent_ack_and_future_receive_failures_cancel_all_preparations_before_valid_commit() {
    let mut buffers = Buffers::new(4, 3, 32);
    let mut endpoint = buffers.endpoint();
    let _ = endpoint.enqueue(b"out").expect("Queued frame");
    let before = endpoint.state();
    assert_eq!(
        endpoint.ingest(packet(Some(0), 1, true, b"\x01\0\0\0x"), ms(100)),
        Err(EndpointError::Send(SendError::AckUnsent))
    );
    assert_eq!(endpoint.state(), before);
    assert!(
        transmit(&mut endpoint, ms(0))
            .expect("Original time still valid")
            .is_some()
    );
    assert_eq!(
        endpoint.ingest(packet(Some(3), 1, true, b"future"), ms(100)),
        Err(EndpointError::Receive(ReceiveError::OutOfWindow))
    );
    assert_eq!(endpoint.state(), before);
    assert!(
        matches!(endpoint.poll(ms(100)).expect("No invalid force effect"), SchedulePoll::WaitUntil(at) if at == ms(250))
    );
    let accepted = endpoint
        .ingest(packet(Some(0), 1, true, b"\x01\0\0\0x"), ms(100))
        .expect("Whole admission");
    assert_eq!(
        (
            accepted.acknowledgement.released_fragments,
            endpoint.state().outgoing_fragments,
            endpoint.state().incoming_bytes
        ),
        (1, 0, 5)
    );
    let wire = transmit(&mut endpoint, ms(100))
        .expect("Validated force now eligible")
        .expect("ACK");
    assert_eq!(&wire[..4], &(-1_i32).to_le_bytes());
    assert_eq!(&wire[4..8], &0_i32.to_le_bytes());
    assert_eq!(endpoint.drain(&mut [0; 8], 8, 2).usage.bytes, 5);
    assert_eq!(endpoint.frame().expect("Complete incoming").body(), b"x");
}
