// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_reliable::{
    ReliableHeader, ReliablePacket, SelectiveAckRange,
    receive::{MAX_RECEIVE_WINDOW_FRAGMENTS, ReceiveWindow},
};

use crate::{packet, storage};

#[test]
fn selective_pages_are_disjoint_complete_and_relative_to_the_consumed_base() {
    let (mut bytes, mut slots) = storage(20);
    let mut window = ReceiveWindow::new(&mut bytes, &mut slots).expect("Window");
    for sequence in [1, 2, 4, 6, 8, 9, 11, 18, 19] {
        let _ = window
            .prepare(packet(Some(sequence), b"x"))
            .expect("Run")
            .commit();
    }
    let mut offset = 0;
    let mut ranges = Vec::new();
    for _ in 0..3 {
        let page = window.acknowledgement(offset);
        assert_eq!(page.cumulative, 0);
        ranges.extend_from_slice(page.selective.ranges());
        offset = page.next_offset;
        if offset == 0 {
            break;
        }
    }
    assert_eq!(offset, 0);
    let expected = [(1, 2), (4, 1), (6, 1), (8, 2), (11, 1), (18, 2)]
        .map(|(offset, length)| SelectiveAckRange { offset, length });
    assert_eq!(ranges, expected);
    assert_eq!(window.acknowledgement(u16::MAX), window.acknowledgement(0));
    let _ = window
        .prepare(packet(Some(0), b"head"))
        .expect("Head")
        .commit();
    let mut output = [0; 2];
    assert_eq!(window.read(&mut output, 1).bytes, 2);
    assert_eq!(
        window.acknowledgement(0).selective.ranges()[0],
        SelectiveAckRange {
            offset: 0,
            length: 3
        }
    );
    assert_eq!(window.read(&mut output, 1).fragments, 1);
    let page = window.acknowledgement(0);
    assert_eq!(page.cumulative, 1);
    assert_eq!(
        page.selective.ranges()[0],
        SelectiveAckRange {
            offset: 0,
            length: 2
        }
    );
}

#[test]
fn maximum_host_span_produces_signed_short_bounded_metadata_without_remote_allocation() {
    let (mut bytes, mut slots) = storage(MAX_RECEIVE_WINDOW_FRAGMENTS);
    let mut window = ReceiveWindow::new(&mut bytes, &mut slots).expect("Largest host span");
    for sequence in 0..32767 {
        let _ = window
            .prepare(packet(Some(sequence), b""))
            .expect("Empty fragment")
            .commit();
    }
    let page = window.acknowledgement(0);
    assert_eq!(
        page.selective.ranges(),
        &[SelectiveAckRange {
            offset: 0,
            length: 32767
        }]
    );
    assert_eq!(page.next_offset, 0);
    let ack = ReliablePacket::try_new(
        ReliableHeader {
            acknowledge: page.cumulative,
            selective: page.selective,
            ..packet(None, b"").header()
        },
        b"",
    )
    .expect("Representable ACK page");
    let mut encoded = [0; 18];
    assert_eq!(ack.encode(&mut encoded), Ok(18));
    assert_eq!(ReliablePacket::parse(&encoded), Ok(ack));
    assert_eq!(window.buffered_bytes(), 0);
    assert_eq!(window.read(&mut [0; 1], 7).fragments, 7);
    assert_eq!(window.stored_fragments(), 32760);
    assert_eq!(window.read(&mut [0; 1], usize::MAX).fragments, 32760);
    assert_eq!(window.acknowledge(), 32767);
}
