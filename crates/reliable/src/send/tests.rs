// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Direct private boundary setup avoids billions of operations."
)]
use super::{SendError, SendSlot, SendWindow};
use crate::{ReliableHeader, ReliablePacket, SelectiveAcks};

#[test]
fn final_data_sequence_and_cumulative_ack_never_wrap_or_partially_enqueue() {
    let mut bytes = [0xa5; 2400];
    let mut slots = [SendSlot::default(); 2];
    let mut window = SendWindow::new(&mut bytes, &mut slots, 1).expect("Window");
    window.base = 0x7fff_fffe;
    window.next = 0x7fff_fffe;
    assert_eq!(
        window.enqueue(&[0; 1201]),
        Err(SendError::SequenceExhausted)
    );
    assert!(window.bytes.iter().all(|&byte| byte == 0xa5));
    assert!(window.slots.iter().all(|&slot| slot == SendSlot::default()));
    assert_eq!(
        (window.next_sequence(), window.queued_fragments()),
        (0x7fff_fffe, 0)
    );
    let queued = window.enqueue(b"x").expect("Final consumable data");
    assert_eq!(
        (
            queued.first_sequence,
            queued.next_sequence,
            queued.fragments
        ),
        (0x7fff_fffe, 0x7fff_ffff, 1)
    );
    assert_eq!(window.enqueue(b"y"), Err(SendError::SequenceExhausted));
    let tx = window
        .prepare_transmission(0x7fff_fffe)
        .expect("Final data");
    assert_eq!(tx.payload(), b"x");
    let _ = tx.record_sent();
    let ack = ReliablePacket::try_new(
        ReliableHeader {
            sequence: None,
            acknowledge: 0x7fff_ffff,
            flags: 0,
            selective: SelectiveAcks::default(),
            sender_window: 1,
        },
        b"",
    )
    .expect("Final representable ACK");
    let update = window
        .prepare_acknowledgement(ack)
        .expect("Final sent prefix")
        .commit();
    assert_eq!((update.released_fragments, update.released_bytes), (1, 1));
    assert_eq!(
        (
            window.acknowledge(),
            window.queued_fragments(),
            window.buffered_bytes()
        ),
        (0x7fff_ffff, 0, 0)
    );
    assert_eq!(window.enqueue(b"new"), Err(SendError::SequenceExhausted));
    assert_eq!(window.enqueue(b"").expect("Empty no-op").fragments, 0);
    assert_eq!(window.active_sequences(), 0x7fff_ffff..0x7fff_ffff);
}
