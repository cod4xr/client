// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_reliable::send::{SendError, SendWindow};

use crate::{enqueue_fragments, packet, sent, storage};

#[test]
fn queued_but_unsent_prefix_holes_and_selective_claims_fail_even_after_a_higher_fragment_was_sent()
{
    let (mut bytes, mut slots) = storage(4);
    let mut window = SendWindow::new(&mut bytes, &mut slots, 4).expect("Window");
    enqueue_fragments(&mut window, 3);
    sent(&mut window, 2);
    assert_eq!(
        window
            .prepare_acknowledgement(packet(None, 3, &[]))
            .expect_err("Unsent holes"),
        SendError::AckUnsent
    );
    assert_eq!(
        window
            .prepare_acknowledgement(packet(None, 0, &[(1, 1)]))
            .expect_err("Unsent SACK"),
        SendError::AckUnsent
    );
    let update = window
        .prepare_acknowledgement(packet(None, 0, &[(2, 1)]))
        .expect("Sent SACK")
        .commit();
    assert_eq!((update.released_fragments, update.newly_selective), (0, 1));
    assert_eq!(
        window.prepare_transmission(2).expect_err("SACK retained"),
        SendError::Selective
    );
    assert_eq!(window.buffered_bytes(), 3);
    sent(&mut window, 0);
    assert!(
        !window
            .prepare_transmission(0)
            .expect("Same original ID")
            .record_sent()
            .first_transmission
    );
    sent(&mut window, 1);
    assert_eq!(
        window
            .prepare_acknowledgement(packet(None, 3, &[]))
            .expect("Actual sent prefix")
            .commit()
            .released_bytes,
        3
    );
    assert_eq!((window.acknowledge(), window.queued_fragments()), (3, 0));
}

#[test]
fn late_invalid_range_cannot_release_a_valid_prefix_or_apply_an_earlier_valid_selective_range() {
    let (mut bytes, mut slots) = storage(4);
    let mut window = SendWindow::new(&mut bytes, &mut slots, 4).expect("Window");
    enqueue_fragments(&mut window, 4);
    sent(&mut window, 0);
    sent(&mut window, 1);
    assert_eq!(
        window
            .prepare_acknowledgement(packet(None, 1, &[(0, 1), (1, 1)]))
            .expect_err("Later unsent"),
        SendError::AckUnsent
    );
    assert_eq!(
        (
            window.acknowledge(),
            window.queued_fragments(),
            window.buffered_bytes()
        ),
        (0, 4, 4)
    );
    assert!(
        !window
            .prepare_transmission(1)
            .expect("Earlier SACK not applied")
            .record_sent()
            .first_transmission
    );
    let prepared = window
        .prepare_acknowledgement(packet(None, 1, &[(0, 1)]))
        .expect("Whole valid ACK");
    assert_eq!(
        (
            prepared.summary().released_fragments,
            prepared.summary().newly_selective
        ),
        (1, 1)
    );
    drop(prepared);
    assert_eq!((window.acknowledge(), window.queued_fragments()), (0, 4));
    let _ = window
        .prepare_transmission(1)
        .expect("Cancelled SACK not applied")
        .record_sent();
}

#[test]
fn unsorted_overlapping_ranges_deduplicate_new_marks_but_retain_all_bytes_until_consumed() {
    let (mut bytes, mut slots) = storage(5);
    let mut window = SendWindow::new(&mut bytes, &mut slots, 4).expect("Window");
    enqueue_fragments(&mut window, 5);
    for sequence in 0..4 {
        sent(&mut window, sequence);
    }
    let claims = [(3, 1), (1, 3), (2, 1)];
    let update = window
        .prepare_acknowledgement(packet(None, 0, &claims))
        .expect("Overlapping ranges")
        .commit();
    assert_eq!(update.newly_selective, 3);
    assert_eq!((window.queued_fragments(), window.buffered_bytes()), (5, 5));
    assert_eq!(
        window
            .prepare_acknowledgement(packet(None, 0, &claims))
            .expect("Repeated")
            .commit()
            .newly_selective,
        0
    );
    for sequence in 1..4 {
        assert_eq!(
            window
                .prepare_transmission(sequence)
                .expect_err("Suppressed"),
            SendError::Selective
        );
    }
    assert_eq!(window.enqueue(b"x"), Err(SendError::Capacity));
    assert_eq!(
        window
            .prepare_transmission(4)
            .expect_err("Beyond current span"),
        SendError::OutsideWindow
    );
    let _ = window
        .prepare_acknowledgement(packet(None, 1, &[]))
        .expect("Consume first")
        .commit();
    assert_eq!(
        window
            .prepare_transmission(1)
            .expect_err("Omitted SACK persists"),
        SendError::Selective
    );
    sent(&mut window, 4);
    let update = window
        .prepare_acknowledgement(packet(None, 5, &[]))
        .expect("Consume remaining")
        .commit();
    assert_eq!((update.released_fragments, update.released_bytes), (4, 4));
    assert_eq!(
        (
            window.acknowledge(),
            window.queued_fragments(),
            window.buffered_bytes()
        ),
        (5, 0, 0)
    );
}

#[test]
fn ack_domains_check_queued_end_active_span_and_regression_without_peer_hint_authority() {
    let (mut bytes, mut slots) = storage(8);
    let mut window = SendWindow::new(&mut bytes, &mut slots, 4).expect("Window");
    enqueue_fragments(&mut window, 8);
    for sequence in 0..4 {
        sent(&mut window, sequence);
    }
    assert_eq!(
        window
            .prepare_acknowledgement(packet(None, 5, &[]))
            .expect_err("Active span"),
        SendError::AckFuture
    );
    for ranges in [&[(3, 2)][..], &[(100, 1)], &[(32767, 32767)]] {
        assert_eq!(
            window
                .prepare_acknowledgement(packet(None, 0, ranges))
                .expect_err("Range span"),
            SendError::AckRange
        );
    }
    assert_eq!(
        window
            .prepare_acknowledgement(packet(None, 4, &[(0, 1)]))
            .expect_err("Next window queued but not sent"),
        SendError::AckUnsent
    );
    let _ = window
        .prepare_acknowledgement(packet(None, 4, &[]))
        .expect("First flight")
        .commit();
    assert_eq!(window.in_flight_limit(), 4);
    assert_eq!(
        window
            .prepare_acknowledgement(packet(None, 3, &[]))
            .expect_err("Old ACK"),
        SendError::AckRegression
    );
    assert_eq!(
        window
            .prepare_acknowledgement(packet(None, 9, &[]))
            .expect_err("Queued end"),
        SendError::AckFuture
    );
    assert_eq!(
        window
            .prepare_acknowledgement(packet(None, 4, &[(3, 2)]))
            .expect_err("Queued end"),
        SendError::AckRange
    );
    assert_eq!(
        (
            window.acknowledge(),
            window.queued_fragments(),
            window.buffered_bytes()
        ),
        (4, 4, 4)
    );
}
