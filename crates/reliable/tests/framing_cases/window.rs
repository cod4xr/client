// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_reliable::{
    ReliableHeader, ReliablePacket, SelectiveAcks,
    framing::{FrameAssembler, NATIVE_MESSAGE_BODY_BYTES, NATIVE_MESSAGE_BYTES},
    receive::{ReceiveSlot, ReceiveWindow},
};

#[test]
fn native_max_message_drains_incrementally_through_three_reordered_slots_without_deadlock() {
    let expected: Vec<u8> = (0..NATIVE_MESSAGE_BODY_BYTES)
        .map(|index| index.to_le_bytes()[0])
        .collect();
    let mut source = i32::try_from(expected.len())
        .expect("Native length")
        .to_le_bytes()
        .to_vec();
    source.extend_from_slice(&expected);
    assert_eq!(source.len(), NATIVE_MESSAGE_BYTES);
    let mut message_storage = vec![0xa5; NATIVE_MESSAGE_BODY_BYTES];
    let mut assembler = FrameAssembler::new(&mut message_storage).expect("Explicit native budget");
    let mut fragment_bytes = [0xa5; 3600];
    let mut slots = [ReceiveSlot::default(); 3];
    let mut window =
        ReceiveWindow::new(&mut fragment_bytes, &mut slots).expect("Small receive span");
    let mut sequence = 0_u32;
    let mut transferred = 0;
    for batch in source.chunks(3600) {
        let fragments: Vec<&[u8]> = batch.chunks(1200).collect();
        for (index, payload) in fragments.iter().enumerate().rev() {
            let header = ReliableHeader {
                sequence: Some(sequence + u32::try_from(index).expect("Three slots")),
                acknowledge: 0,
                flags: 0,
                selective: SelectiveAcks::default(),
                sender_window: 4,
            };
            let packet = ReliablePacket::try_new(header, payload).expect("Wire fragment");
            let _ = window
                .prepare(packet)
                .expect("Reordered preflight")
                .commit();
        }
        sequence += u32::try_from(fragments.len()).expect("Three slots");
        while window.stored_fragments() != 0 {
            let mut output = [0; 97];
            let read = window.read(&mut output, 2);
            assert!(read.bytes > 0);
            let progress = assembler
                .push(&output[..read.bytes])
                .expect("Incremental application bytes");
            assert_eq!(progress.consumed, read.bytes);
            transferred += progress.consumed;
            assert_eq!(progress.complete, transferred == source.len());
        }
        assert_eq!(window.acknowledge(), sequence);
    }
    assert_eq!(transferred, source.len());
    assert!(assembler.frame().expect("Native maximum frame").body() == expected);
    assert_eq!((window.stored_fragments(), window.buffered_bytes()), (0, 0));
}
