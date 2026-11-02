// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{Buffers, ms, packet};
use cod4xr_reliable::{
    endpoint::{EndpointConfig, EndpointError, ReliableEndpoint},
    receive::{ReceiveError, ReceiveWindow},
    schedule::{ScheduleConfig, ScheduleError},
    send::SendWindow,
};
use std::time::Duration;

#[test]
fn late_storage_or_timer_failure_preserves_both_live_metadata_arrays_and_dirty_bytes() {
    let mut buffers = Buffers::new(4, 3, 32);
    {
        let mut tx =
            SendWindow::new(&mut buffers.send, &mut buffers.send_slots, 4).expect("Existing queue");
        let _ = tx.enqueue(b"PrivatePrevious").expect("Original");
        let mut rx = ReceiveWindow::new(&mut buffers.receive, &mut buffers.receive_slots)
            .expect("Existing receive");
        let _ = rx
            .prepare(packet(Some(0), 0, false, b"PrivateIncoming"))
            .expect("Slot")
            .commit();
    }
    let original_tx = (buffers.send.clone(), buffers.send_slots.clone());
    let original_rx = (buffers.receive.clone(), buffers.receive_slots.clone());
    let removed = buffers
        .receive
        .pop()
        .expect("Deliberately broken final slot");
    assert_eq!(
        ReliableEndpoint::new(ms(0), EndpointConfig::default(), buffers.storage())
            .expect_err("Late invalid geometry"),
        EndpointError::Receive(ReceiveError::Storage)
    );
    assert_eq!(
        (&buffers.send, &buffers.send_slots),
        (&original_tx.0, &original_tx.1)
    );
    buffers.receive.push(removed);
    assert_eq!(
        (&buffers.receive, &buffers.receive_slots),
        (&original_rx.0, &original_rx.1)
    );
    let bad_timing = EndpointConfig {
        schedule: ScheduleConfig {
            data_interval: Duration::ZERO,
            ..ScheduleConfig::default()
        },
        ..EndpointConfig::default()
    };
    assert_eq!(
        ReliableEndpoint::new(ms(0), bad_timing, buffers.storage()).expect_err("Timing"),
        EndpointError::Schedule(ScheduleError::Configuration)
    );
    assert_eq!(
        ReliableEndpoint::new(Duration::MAX, EndpointConfig::default(), buffers.storage())
            .expect_err("Timer overflow"),
        EndpointError::Schedule(ScheduleError::ClockOverflow)
    );
    assert_eq!(
        (&buffers.send, &buffers.send_slots),
        (&original_tx.0, &original_tx.1)
    );
    assert_eq!(
        (&buffers.receive, &buffers.receive_slots),
        (&original_rx.0, &original_rx.1)
    );
    assert!(buffers.body.iter().all(|&byte| byte == 0xa5));
    let endpoint = buffers.endpoint();
    assert_eq!(
        (
            endpoint.state().incoming_fragments,
            endpoint.state().outgoing_fragments
        ),
        (0, 0)
    );
    assert!(!format!("{endpoint:?}").contains("Private"));
}
