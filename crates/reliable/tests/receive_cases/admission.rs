// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_reliable::{
    ReliableHeader, ReliablePacket,
    receive::{
        MAX_RECEIVE_WINDOW_FRAGMENTS, NATIVE_RECEIVE_WINDOW_FRAGMENTS, ReceiveDisposition,
        ReceiveError, ReceiveSlot, ReceiveWindow, required_receive_bytes,
    },
};

use crate::{packet, storage};

#[test]
fn geometry_is_exact_and_invalid_construction_preserves_live_metadata_and_dirty_bytes() {
    assert_eq!(NATIVE_RECEIVE_WINDOW_FRAGMENTS, 20);
    for capacity in [0, MAX_RECEIVE_WINDOW_FRAGMENTS + 1, usize::MAX] {
        assert_eq!(required_receive_bytes(capacity), Err(ReceiveError::Storage));
    }
    let (mut bytes, mut slots) = storage(2);
    {
        let mut window = ReceiveWindow::new(&mut bytes, &mut slots).expect("Window");
        assert_eq!(window.capacity(), 2);
        assert_eq!(window.buffered_bytes(), 0);
        let mut output = [0x99; 12];
        assert_eq!(window.read(&mut output, usize::MAX).bytes, 0);
        assert_eq!(output, [0x99; 12]);
        let _ = window
            .prepare(packet(Some(0), b"PrivateOriginal"))
            .expect("Data")
            .commit();
    }
    let before_bytes = bytes.clone();
    let before_slots = slots.clone();
    for length in [0, 1, 2399] {
        assert_eq!(
            ReceiveWindow::new(&mut bytes[..length], &mut slots).expect_err("Geometry"),
            ReceiveError::Storage
        );
        assert_eq!(bytes, before_bytes);
        assert_eq!(slots, before_slots);
    }
    let window = ReceiveWindow::new(&mut bytes, &mut slots).expect("Fresh metadata");
    assert_eq!((window.stored_fragments(), window.buffered_bytes()), (0, 0));
    assert!(window.acknowledgement(0).selective.is_empty());
    assert_eq!(slots, vec![ReceiveSlot::default(); 2]);
    assert_eq!(bytes, before_bytes);
}

#[test]
fn cancelled_preparation_and_outside_window_errors_leave_fragments_and_output_unchanged() {
    let (mut bytes, mut slots) = storage(3);
    let mut window = ReceiveWindow::new(&mut bytes, &mut slots).expect("Window");
    let pending = window
        .prepare(packet(Some(1), b"PrivatePayload"))
        .expect("Preflight");
    assert_eq!(
        pending.summary(),
        ReceiveDisposition::Stored {
            sequence: 1,
            bytes: 14
        }
    );
    assert!(!format!("{pending:?}").contains("PrivatePayload"));
    drop(pending); // e.g. separate outgoing ACK validation failed.
    assert_eq!((window.stored_fragments(), window.buffered_bytes()), (0, 0));
    for sequence in [3, 4, 100] {
        assert_eq!(
            window
                .prepare(packet(Some(sequence), b"future"))
                .expect_err("Window"),
            ReceiveError::OutOfWindow
        );
    }
    assert_eq!(window.acknowledge(), 0);
    assert!(window.acknowledgement(0).selective.is_empty());
    assert!(bytes.iter().all(|&byte| byte == 0xa5));
    assert_eq!(slots, vec![ReceiveSlot::default(); 3]);
}

#[test]
fn duplicate_partial_and_old_sequences_cannot_overwrite_original_stream_bytes() {
    let (mut bytes, mut slots) = storage(2);
    let mut window = ReceiveWindow::new(&mut bytes, &mut slots).expect("Window");
    let _ = window
        .prepare(packet(Some(0), b"PrivateOriginal"))
        .expect("Data")
        .commit();
    let mut output = [0; 7];
    assert_eq!(window.read(&mut output, 1).bytes, 7);
    assert_eq!(&output, b"Private");
    assert_eq!(window.acknowledge(), 0);
    assert_eq!(window.stored_fragments(), 1);
    assert_eq!(window.buffered_bytes(), 8);
    assert_eq!(
        window
            .prepare(packet(Some(0), b"replaced"))
            .expect("Duplicate")
            .commit(),
        ReceiveDisposition::Duplicate { sequence: 0 }
    );
    let mut output = [0xa5; 20];
    assert_eq!(window.read(&mut output, 1).bytes, 8);
    assert_eq!(&output[..8], b"Original");
    assert_eq!(&output[8..], &[0xa5; 12]);
    assert_eq!(window.acknowledge(), 1);
    assert_eq!(
        window
            .prepare(packet(Some(0), b"old"))
            .expect("Old")
            .commit(),
        ReceiveDisposition::Old { sequence: 0 }
    );
    assert_eq!((window.stored_fragments(), window.buffered_bytes()), (0, 0));
    assert!(!format!("{window:?}").contains("PrivateOriginal"));
}

#[test]
fn receive_preflight_never_applies_unvalidated_peer_acks_flags_or_window_hints() {
    let (mut bytes, mut slots) = storage(1);
    let mut window = ReceiveWindow::new(&mut bytes, &mut slots).expect("Window");
    let fields = ReliableHeader {
        acknowledge: 0x7fff_ffff,
        flags: u8::MAX,
        sender_window: i16::MAX,
        ..packet(None, b"").header()
    };
    let body = ReliablePacket::try_new(fields, b"").expect("Unvalidated metadata");
    assert_eq!(
        window.prepare(body).expect("Receive side only").commit(),
        ReceiveDisposition::AckOnly
    );
    assert_eq!(
        (
            window.capacity(),
            window.acknowledge(),
            window.stored_fragments()
        ),
        (1, 0, 0)
    );
    assert!(window.acknowledgement(0).selective.is_empty());
}
