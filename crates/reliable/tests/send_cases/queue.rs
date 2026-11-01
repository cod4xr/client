// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_reliable::{
    ReliablePacket, SelectiveAcks,
    send::{SendError, SendSlot, SendWindow, required_send_bytes},
};

use crate::{enqueue_fragments, packet, sent, storage};

#[test]
fn invalid_geometry_and_unfunded_spans_preserve_original_bytes_and_live_metadata() {
    assert_eq!(required_send_bytes(0), Err(SendError::Storage));
    assert_eq!(required_send_bytes(usize::MAX), Err(SendError::Storage));
    let (mut bytes, mut slots) = storage(4);
    {
        let mut window = SendWindow::new(&mut bytes, &mut slots, 4).expect("Window");
        assert_eq!(
            window.enqueue(b"PrivateOriginal").expect("Owned").fragments,
            1
        );
    }
    let before_bytes = bytes.clone();
    let before_slots = slots.clone();
    for span in [0, 5, 32768, u16::MAX] {
        assert_eq!(
            SendWindow::new(&mut bytes, &mut slots, span).expect_err("Span"),
            SendError::Storage
        );
        assert_eq!(bytes, before_bytes);
        assert_eq!(slots, before_slots);
    }
    assert_eq!(
        SendWindow::new(&mut bytes[..4799], &mut slots, 4).expect_err("Exact bytes"),
        SendError::Storage
    );
    assert_eq!(bytes, before_bytes);
    assert_eq!(slots, before_slots);
    let window = SendWindow::new(&mut bytes, &mut slots, 1).expect("Fresh metadata");
    assert_eq!(
        (
            window.capacity(),
            window.in_flight_limit(),
            window.queued_fragments(),
            window.buffered_bytes()
        ),
        (4, 1, 0, 0)
    );
    assert_eq!(window.active_sequences(), 0..0);
    assert_eq!(slots, vec![SendSlot::default(); 4]);
    assert_eq!(bytes, before_bytes);
}

#[test]
fn exact_fragment_boundaries_copy_original_binary_data_without_empty_terminators() {
    for length in [0_usize, 1, 1199, 1200, 1201, 2400] {
        let (mut bytes, mut slots) = storage(4);
        let mut window = SendWindow::new(&mut bytes, &mut slots, 4).expect("Window");
        let mut source: Vec<u8> = (0..length).map(|index| index.to_le_bytes()[0]).collect();
        let expected = source.clone();
        let queued = window.enqueue(&source).expect("Complete enqueue");
        source.fill(0xee);
        let count = length.div_ceil(1200);
        assert_eq!(
            (
                queued.first_sequence,
                queued.next_sequence,
                queued.fragments,
                queued.bytes
            ),
            (0, u32::try_from(count).expect("Count"), count, length)
        );
        assert_eq!(
            (window.queued_fragments(), window.buffered_bytes()),
            (count, length)
        );
        let mut actual = Vec::new();
        for sequence in window.active_sequences() {
            let tx = window
                .prepare_transmission(sequence)
                .expect("Original bytes");
            actual.extend_from_slice(tx.payload());
            assert_eq!(tx.sequence(), sequence);
            let mut wire = [0; 1226];
            let length = tx
                .packet(0, SelectiveAcks::default(), 0)
                .expect("Body")
                .encode(&mut wire)
                .expect("Encode");
            let decoded = ReliablePacket::parse(&wire[..length]).expect("Complete codec");
            assert_eq!(decoded.header().sequence, Some(sequence));
            assert_eq!(decoded.header().sender_window, 4);
            assert_eq!(decoded.payload(), tx.payload());
            assert!(tx.record_sent().first_transmission);
        }
        assert_eq!(actual, expected);
        assert_eq!(window.acknowledge(), 0);
    }
}

#[test]
fn queue_capacity_and_in_flight_span_are_independent_and_reused_only_after_cumulative_release() {
    let (mut bytes, mut slots) = storage(3);
    let mut window = SendWindow::new(&mut bytes, &mut slots, 1).expect("Small flight span");
    enqueue_fragments(&mut window, 3);
    assert_eq!(window.active_sequences(), 0..1);
    assert_eq!(
        window.prepare_transmission(1).expect_err("Not active"),
        SendError::OutsideWindow
    );
    assert_eq!(window.enqueue(b"overflow"), Err(SendError::Capacity));
    assert_eq!(
        (
            window.next_sequence(),
            window.buffered_bytes(),
            window.queued_fragments()
        ),
        (3, 3, 3)
    );
    sent(&mut window, 0);
    assert_eq!(
        window
            .prepare_acknowledgement(packet(None, 1, &[]))
            .expect("Consumed")
            .commit()
            .released_fragments,
        1
    );
    assert_eq!(window.enqueue(b"fresh").expect("Reuse").first_sequence, 3);
    assert_eq!(
        window.prepare_transmission(0).expect_err("Released ID"),
        SendError::Missing
    );
    for sequence in 1..4 {
        assert_eq!(window.active_sequences(), sequence..sequence + 1);
        let tx = window.prepare_transmission(sequence).expect("Active");
        let expected = if sequence == 3 {
            b"fresh".as_slice()
        } else if sequence == 1 {
            &[1]
        } else {
            &[2]
        };
        assert_eq!(tx.payload(), expected);
        let _ = tx.record_sent();
        let _ = window
            .prepare_acknowledgement(packet(None, sequence + 1, &[]))
            .expect("Release")
            .commit();
    }
    assert_eq!(
        (
            window.acknowledge(),
            window.queued_fragments(),
            window.buffered_bytes()
        ),
        (4, 0, 0)
    );
}
