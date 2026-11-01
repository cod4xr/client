// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Direct boundary setup avoids billions of iterations."
)]
use super::{ReceiveDisposition, ReceiveError, ReceiveSlot, ReceiveWindow};
use crate::{ReliableHeader, ReliablePacket, SelectiveAcks};

#[test]
fn final_consumable_sequence_advances_to_max_ack_without_wrapping_or_admitting_max_data() {
    let mut bytes = [0xa5; 1200];
    let mut slots = [ReceiveSlot::default(); 1];
    let mut window = ReceiveWindow::new(&mut bytes, &mut slots).expect("Window");
    window.base = 0x7fff_fffe;
    let header = ReliableHeader {
        sequence: Some(0x7fff_fffe),
        acknowledge: 0,
        flags: 0,
        selective: SelectiveAcks::default(),
        sender_window: 4,
    };
    let packet = ReliablePacket::try_new(header, b"x").expect("Last consumable data");
    assert_eq!(
        window.prepare(packet).expect("Preflight").commit(),
        ReceiveDisposition::Stored {
            sequence: 0x7fff_fffe,
            bytes: 1,
        }
    );
    assert_eq!(window.read(&mut [0; 1], 1).fragments, 1);
    assert_eq!(window.acknowledge(), 0x7fff_ffff);
    let final_packet = ReliablePacket::try_new(
        ReliableHeader {
            sequence: Some(0x7fff_ffff),
            ..header
        },
        b"y",
    )
    .expect("Wire representation only");
    assert_eq!(
        window.prepare(final_packet).expect_err("Reconnect"),
        ReceiveError::SequenceExhausted
    );
    assert_eq!(
        (
            window.acknowledge(),
            window.stored_fragments(),
            window.buffered_bytes()
        ),
        (0x7fff_ffff, 0, 0)
    );
    assert!(window.acknowledgement(0).selective.is_empty());
    assert_eq!(
        window.prepare(packet).expect("Old fragment").commit(),
        ReceiveDisposition::Old {
            sequence: 0x7fff_fffe
        }
    );
    let ack_only = ReliablePacket::try_new(
        ReliableHeader {
            sequence: None,
            ..header
        },
        b"",
    )
    .expect("ACK sentinel");
    assert_eq!(
        window
            .prepare(ack_only)
            .expect("ACK remains representable")
            .commit(),
        ReceiveDisposition::AckOnly
    );
}
