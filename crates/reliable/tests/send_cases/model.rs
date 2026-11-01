// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_reliable::{
    ReliableHeader, ReliablePacket, SelectiveAcks,
    receive::{ReceiveSlot, ReceiveWindow},
    send::{SendError, SendWindow},
};

use crate::storage;

#[test]
fn independent_loss_reorder_duplicate_model_preserves_a_binary_stream_across_queue_wraparound() {
    let source: Vec<u8> = (0_usize..50_000)
        .map(|index| index.to_le_bytes()[0])
        .collect();
    let mut admitted = 0;
    let mut actual = Vec::new();
    let (mut send_bytes, mut send_slots) = storage(7);
    let mut outgoing =
        SendWindow::new(&mut send_bytes, &mut send_slots, 4).expect("Outgoing budget");
    let mut receive_bytes = vec![0; 24_000];
    let mut receive_slots = [ReceiveSlot::default(); 20];
    let mut incoming =
        ReceiveWindow::new(&mut receive_bytes, &mut receive_slots).expect("Incoming budget");
    let mut lost = 0;
    let mut duplicates = 0;
    let mut retried = 0;
    for _ in 0..2000 {
        while outgoing.queued_fragments() < outgoing.capacity() && admitted < source.len() {
            let length = (113 + admitted % 1000).min(source.len() - admitted);
            assert_eq!(
                outgoing
                    .enqueue(&source[admitted..admitted + length])
                    .expect("Available slot")
                    .fragments,
                1
            );
            admitted += length;
        }
        for sequence in outgoing.active_sequences().rev() {
            let tx = match outgoing.prepare_transmission(sequence) {
                Ok(tx) => tx,
                Err(error) => {
                    assert_eq!(error, SendError::Selective);
                    continue;
                }
            };
            let mut encoded = [0; 1226];
            let length = tx
                .packet(0, SelectiveAcks::default(), 0)
                .expect("Original body")
                .encode(&mut encoded)
                .expect("Encode");
            let sent = tx.record_sent();
            if sent.first_transmission && sequence % 5 == 1 {
                lost += 1;
                continue; // Successful OS admission, subsequently lost on the simulated wire.
            }
            if !sent.first_transmission {
                retried += 1;
            }
            let packet = ReliablePacket::parse(&encoded[..length]).expect("Exact received body");
            let _ = incoming
                .prepare(packet)
                .expect("Reordered receive")
                .commit();
            if sequence % 7 == 3 {
                let _ = incoming
                    .prepare(packet)
                    .expect("Duplicate receive")
                    .commit();
                duplicates += 1;
            }
        }
        for _ in 0..16 {
            let mut output = [0; 97];
            let progress = incoming.read(&mut output, 2);
            actual.extend_from_slice(&output[..progress.bytes]);
            if progress.examined == 0 {
                break;
            }
        }
        acknowledge_received(&mut outgoing, &incoming);
        if admitted == source.len() && outgoing.queued_fragments() == 0 {
            break;
        }
    }
    assert_eq!(actual, source);
    assert_eq!(
        (
            admitted,
            outgoing.queued_fragments(),
            outgoing.buffered_bytes()
        ),
        (source.len(), 0, 0)
    );
    assert_eq!(
        (incoming.stored_fragments(), incoming.buffered_bytes()),
        (0, 0)
    );
    assert_eq!(incoming.acknowledge(), outgoing.next_sequence());
    assert!(lost > 0 && retried > 0 && duplicates > 0);
}

fn acknowledge_received(outgoing: &mut SendWindow<'_>, incoming: &ReceiveWindow<'_>) {
    let received = incoming.acknowledgement(0);
    let ack = ReliablePacket::try_new(
        ReliableHeader {
            sequence: None,
            acknowledge: received.cumulative,
            flags: 0,
            selective: received.selective,
            sender_window: 4,
        },
        b"",
    )
    .expect("ACK-only");
    let mut wire = [0; 26];
    let length = ack.encode(&mut wire).expect("ACK encode");
    let ack = ReliablePacket::parse(&wire[..length]).expect("ACK decode");
    let _ = outgoing
        .prepare_acknowledgement(ack)
        .expect("Whole actual-send ACK admission")
        .commit();
}
