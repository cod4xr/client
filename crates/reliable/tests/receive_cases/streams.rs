// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_reliable::receive::{ReadProgress, ReceiveWindow};

use crate::{packet, storage};

#[test]
fn maximum_binary_fragment_survives_incremental_reads_and_conflicting_retransmissions() {
    let (mut bytes, mut slots) = storage(1);
    let mut window = ReceiveWindow::new(&mut bytes, &mut slots).expect("Window");
    let original: Vec<u8> = (0_u16..1200).map(|index| index.to_le_bytes()[0]).collect();
    let _ = window
        .prepare(packet(Some(0), &original))
        .expect("Maximum data")
        .commit();
    let mut actual = Vec::new();
    while window.stored_fragments() != 0 {
        let mut output = [0xa5; 17];
        let progress = window.read(&mut output, 1);
        actual.extend_from_slice(&output[..progress.bytes]);
        if window.stored_fragments() != 0 {
            assert_eq!(window.acknowledge(), 0);
            assert_eq!(window.acknowledgement(0).selective.ranges()[0].length, 1);
            let _ = window
                .prepare(packet(Some(0), &[0xff; 1200]))
                .expect("Duplicate")
                .commit();
        }
    }
    assert_eq!(actual, original);
    assert_eq!(window.acknowledge(), 1);
    assert_eq!(window.buffered_bytes(), 0);
}

#[test]
fn reordered_chunks_match_an_independent_concatenation_across_repeated_slot_reuse() {
    for capacity in [1, 2, 3, 20, 256] {
        let (mut bytes, mut slots) = storage(capacity);
        let mut window = ReceiveWindow::new(&mut bytes, &mut slots).expect("Window");
        let mut expected = Vec::new();
        let mut actual = Vec::new();
        let mut sequence = 0_u32;
        for _ in 0..9 {
            let first = sequence;
            for _ in 0..capacity {
                let length = usize::try_from(sequence % 11).expect("Finite length");
                expected.extend_from_slice(&[sequence.to_le_bytes()[0]; 10][..length]);
                sequence += 1;
            }
            for current in (first..sequence).rev() {
                let length = usize::try_from(current % 11).expect("Finite length");
                let payload = [current.to_le_bytes()[0]; 10];
                let _ = window
                    .prepare(packet(Some(current), &payload[..length]))
                    .expect("Reorder")
                    .commit();
            }
            while window.stored_fragments() != 0 {
                let mut output = [0xa5; 5];
                let progress = window.read(&mut output, 2);
                assert!(progress.examined <= 2);
                assert!(progress.fragments <= progress.examined);
                assert!(progress.examined > 0);
                assert_eq!(&output[progress.bytes..], &[0xa5; 5][..5 - progress.bytes]);
                actual.extend_from_slice(&output[..progress.bytes]);
            }
            assert_eq!(window.acknowledge(), sequence);
            assert_eq!(window.buffered_bytes(), 0);
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn gaps_output_capacity_and_empty_fragment_work_quotas_each_stop_reads() {
    let (mut bytes, mut slots) = storage(5);
    let mut window = ReceiveWindow::new(&mut bytes, &mut slots).expect("Window");
    let _ = window
        .prepare(packet(Some(4), b"tail"))
        .expect("Reordered")
        .commit();
    let mut output = [0xa5; 5];
    assert_eq!(
        window.read(&mut output, usize::MAX),
        ReadProgress::default()
    );
    assert_eq!(output, [0xa5; 5]);
    for sequence in 0..4 {
        let _ = window
            .prepare(packet(Some(sequence), b""))
            .expect("Empty")
            .commit();
    }
    assert_eq!(window.read(&mut output, 0), ReadProgress::default());
    assert_eq!(window.read(&mut [], 5), ReadProgress::default());
    assert_eq!(window.acknowledge(), 0);
    assert_eq!(
        window.read(&mut output, 2),
        ReadProgress {
            bytes: 0,
            fragments: 2,
            examined: 2
        }
    );
    assert_eq!(window.acknowledge(), 2);
    assert_eq!(
        window.read(&mut output, 2),
        ReadProgress {
            bytes: 0,
            fragments: 2,
            examined: 2
        }
    );
    assert_eq!(window.acknowledge(), 4);
    assert_eq!(
        window.read(&mut output[..2], usize::MAX),
        ReadProgress {
            bytes: 2,
            fragments: 0,
            examined: 1
        }
    );
    assert_eq!(window.acknowledge(), 4);
    assert_eq!(&output[..2], b"ta");
    assert_eq!(
        window.read(&mut output, 1),
        ReadProgress {
            bytes: 2,
            fragments: 1,
            examined: 1
        }
    );
    assert_eq!(&output[..2], b"il");
    assert_eq!(window.acknowledge(), 5);
}
