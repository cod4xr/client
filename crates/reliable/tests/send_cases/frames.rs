// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::storage;
use cod4xr_reliable::{
    framing::ReliableFrame,
    send::{SendError, SendSlot, SendWindow},
};

#[test]
fn framed_enqueue_copies_prefix_and_binary_body_once_across_every_fragment_boundary() {
    for length in [0_usize, 1, 1195, 1196, 1197, 2396] {
        let (mut bytes, mut slots) = storage(4);
        let mut window = SendWindow::new(&mut bytes, &mut slots, 4).expect("Window");
        let mut source: Vec<u8> = (0..length).map(|index| index.to_le_bytes()[0]).collect();
        let mut expected = i32::try_from(length)
            .expect("Length")
            .to_le_bytes()
            .to_vec();
        expected.extend_from_slice(&source);
        let queued = window
            .enqueue_frame(ReliableFrame::new(&source).expect("Body"))
            .expect("Atomic frame");
        source.fill(0xee);
        assert_eq!(
            (queued.bytes, queued.fragments),
            (length + 4, (length + 4).div_ceil(1200))
        );
        let mut actual = Vec::new();
        for sequence in window.active_sequences() {
            let tx = window.prepare_transmission(sequence).expect("Original");
            actual.extend_from_slice(tx.payload());
        }
        assert_eq!(actual, expected);
        assert_eq!(window.buffered_bytes(), length + 4);
    }
}

#[test]
fn complete_frame_capacity_is_checked_before_queuing_its_prefix_or_overwriting_original_data() {
    let (mut bytes, mut slots) = storage(1);
    {
        let mut window = SendWindow::new(&mut bytes, &mut slots, 1).expect("Window");
        let too_large = [0xfe; 1197];
        assert_eq!(
            window.enqueue_frame(ReliableFrame::new(&too_large).expect("Body")),
            Err(SendError::Capacity)
        );
        assert_eq!(
            (
                window.next_sequence(),
                window.queued_fragments(),
                window.buffered_bytes()
            ),
            (0, 0, 0)
        );
    }
    assert!(bytes.iter().all(|&byte| byte == 0xa5));
    assert_eq!(slots, [SendSlot::default()]);
    let mut window = SendWindow::new(&mut bytes, &mut slots, 1).expect("Window");
    let body = [0xfe; 1196];
    let _ = window
        .enqueue_frame(ReliableFrame::new(&body).expect("Body"))
        .expect("Exact slot");
    assert_eq!(
        window.enqueue_frame(ReliableFrame::new(b"").expect("Empty body")),
        Err(SendError::Capacity)
    );
    let tx = window
        .prepare_transmission(0)
        .expect("Original frame retained");
    assert_eq!(&tx.payload()[..4], &1196_i32.to_le_bytes());
    assert_eq!(&tx.payload()[4..], &body);
}
