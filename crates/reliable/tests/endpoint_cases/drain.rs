// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{Buffers, ms, packet};
use cod4xr_reliable::{
    endpoint::{DrainStatus, DrainUsage, EndpointError},
    framing::FrameError,
};

#[test]
fn gap_empty_fragment_quotas_and_concatenated_frames_never_consume_an_uninspected_next_body() {
    let mut buffers = Buffers::new(4, 3, 32);
    let mut endpoint = buffers.endpoint();
    let stream = b"\x03\0\0\0one\x03\0\0\0two";
    let _ = endpoint
        .ingest(packet(Some(2), 0, false, stream), ms(0))
        .expect("Reordered");
    assert_eq!(
        endpoint.drain(&mut [0; 8], 100, 10).usage,
        DrainUsage::default()
    );
    for sequence in 0..2 {
        let _ = endpoint
            .ingest(packet(Some(sequence), 0, false, b""), ms(0))
            .expect("Empty fragment");
    }
    assert_eq!(
        endpoint.drain(&mut [], 100, 10).usage,
        DrainUsage::default()
    );
    assert_eq!(
        endpoint.drain(&mut [0; 8], 0, 10).usage,
        DrainUsage::default()
    );
    assert_eq!(
        endpoint.drain(&mut [0; 8], 100, 0).usage,
        DrainUsage::default()
    );
    for base in [1, 2] {
        let drain = endpoint.drain(&mut [0; 8], 100, 1);
        assert_eq!(
            drain.usage,
            DrainUsage {
                bytes: 0,
                fragments: 1,
                examined: 1
            }
        );
        assert_eq!(drain.outcome, Ok(DrainStatus::Pending));
        assert_eq!(endpoint.state().incoming_acknowledge, base);
    }
    assert_eq!(endpoint.drain(&mut [0; 8], 2, 10).usage.bytes, 2);
    assert!(endpoint.frame().is_none());
    let first = endpoint.drain(&mut [0; 8], 100, 10);
    assert_eq!(
        (first.usage.bytes, first.outcome),
        (5, Ok(DrainStatus::Ready))
    );
    assert_eq!(endpoint.frame().expect("First only").body(), b"one");
    assert_eq!(
        (
            endpoint.state().incoming_bytes,
            endpoint.state().incoming_acknowledge
        ),
        (7, 2)
    );
    assert_eq!(
        endpoint.drain(&mut [0; 8], usize::MAX, usize::MAX).usage,
        DrainUsage::default()
    );
    assert!(endpoint.consume_frame());
    let second = endpoint.drain(&mut [0; 8], 100, 10);
    assert_eq!(
        (second.usage.bytes, second.outcome),
        (7, Ok(DrainStatus::Ready))
    );
    assert_eq!(endpoint.frame().expect("Second retained").body(), b"two");
    assert_eq!(
        (
            endpoint.state().incoming_bytes,
            endpoint.state().incoming_acknowledge
        ),
        (0, 3)
    );
}

#[test]
fn invalid_length_preserves_failure_work_counters_and_permanently_stops_endpoint_admission() {
    for (declared, expected) in [(-1_i32, FrameError::Length), (4, FrameError::BodyCapacity)] {
        let mut buffers = Buffers::new(4, 3, 3);
        let mut endpoint = buffers.endpoint();
        let mut input = declared.to_le_bytes().to_vec();
        input.extend_from_slice(&[0; 4]);
        let _ = endpoint
            .ingest(packet(Some(0), 0, false, &input), ms(0))
            .expect("Transport bytes");
        let drain = endpoint.drain(&mut [0; 64], 64, 8);
        assert_eq!(drain.outcome, Err(expected));
        assert_eq!(
            drain.usage,
            DrainUsage {
                bytes: 4,
                fragments: 0,
                examined: 1
            }
        );
        assert!(endpoint.state().failed);
        assert_eq!(endpoint.state().incoming_bytes, 4);
        assert!(endpoint.frame().is_none());
        assert!(!endpoint.consume_frame());
        let before = endpoint.state();
        assert_eq!(endpoint.enqueue(b"new"), Err(EndpointError::Failed));
        assert_eq!(
            endpoint.poll(ms(1)).expect_err("Terminal"),
            EndpointError::Failed
        );
        assert_eq!(
            endpoint.ingest(packet(None, 0, true, b""), ms(1)),
            Err(EndpointError::Failed)
        );
        let failed = endpoint.drain(&mut [0; 64], 64, 8);
        assert_eq!(
            (failed.outcome, failed.usage),
            (Err(FrameError::Failed), DrainUsage::default())
        );
        assert_eq!(endpoint.state(), before);
    }
}
