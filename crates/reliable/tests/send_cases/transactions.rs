// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_reliable::{
    ReliablePacket,
    receive::{ReceiveError, ReceiveSlot, ReceiveWindow},
    send::{SendError, SendWindow},
};

use crate::{packet, sent, storage};

#[test]
fn cancelled_send_and_ack_preparations_never_grant_unsent_or_released_authority() {
    let (mut bytes, mut slots) = storage(1);
    let mut window = SendWindow::new(&mut bytes, &mut slots, 1).expect("Window");
    let _ = window.enqueue(b"PrivatePayload").expect("Queue");
    let sending = window
        .prepare_transmission(0)
        .expect("No successful I/O yet");
    assert_eq!(sending.payload(), b"PrivatePayload");
    assert!(!format!("{sending:?}").contains("PrivatePayload"));
    drop(sending);
    assert_eq!(
        window
            .prepare_acknowledgement(packet(None, 1, &[]))
            .expect_err("Never sent"),
        SendError::AckUnsent
    );
    sent(&mut window, 0);
    let pending = window
        .prepare_acknowledgement(packet(None, 1, &[]))
        .expect("Valid preflight");
    assert_eq!(
        (
            pending.summary().released_fragments,
            pending.summary().released_bytes
        ),
        (1, 14)
    );
    assert!(!format!("{pending:?}").contains("PrivatePayload"));
    drop(pending);
    assert_eq!(
        (
            window.acknowledge(),
            window.queued_fragments(),
            window.buffered_bytes()
        ),
        (0, 1, 14)
    );
    let tx = window.prepare_transmission(0).expect("Original retained");
    assert_eq!(tx.payload(), b"PrivatePayload");
    assert!(!tx.record_sent().first_transmission);
    assert!(!format!("{window:?}").contains("PrivatePayload"));
}

#[test]
fn paired_receive_and_outgoing_ack_preflight_cancel_together_before_any_partial_mutation() {
    let (mut bytes, mut slots) = storage(1);
    let mut outgoing = SendWindow::new(&mut bytes, &mut slots, 1).expect("Outgoing");
    let _ = outgoing.enqueue(b"old").expect("Queued but not sent");
    let mut receive_bytes = [0xa5; 1200];
    let mut receive_slots = [ReceiveSlot::default(); 1];
    let mut incoming =
        ReceiveWindow::new(&mut receive_bytes, &mut receive_slots).expect("Incoming");
    let data =
        ReliablePacket::try_new(packet(Some(0), 1, &[]).header(), b"new").expect("Valid body");
    let receive_update = incoming.prepare(data).expect("Incoming preflight");
    assert_eq!(
        outgoing
            .prepare_acknowledgement(data)
            .expect_err("Unsent ACK"),
        SendError::AckUnsent
    );
    drop(receive_update);
    assert_eq!(
        (incoming.stored_fragments(), incoming.buffered_bytes()),
        (0, 0)
    );
    assert_eq!(
        (outgoing.acknowledge(), outgoing.queued_fragments()),
        (0, 1)
    );
    sent(&mut outgoing, 0);
    let future = ReliablePacket::try_new(packet(Some(1), 1, &[]).header(), b"future")
        .expect("Wire representation");
    let ack_update = outgoing
        .prepare_acknowledgement(future)
        .expect("ACK alone valid");
    assert_eq!(
        incoming.prepare(future).expect_err("Incoming outside span"),
        ReceiveError::OutOfWindow
    );
    drop(ack_update);
    assert_eq!(
        (outgoing.acknowledge(), outgoing.queued_fragments()),
        (0, 1)
    );
    let receive_update = incoming.prepare(data).expect("Whole receive valid");
    let ack_update = outgoing
        .prepare_acknowledgement(data)
        .expect("Whole ACK valid");
    assert_eq!(ack_update.commit().released_fragments, 1);
    let _ = receive_update.commit();
    assert_eq!(
        (outgoing.acknowledge(), outgoing.queued_fragments()),
        (1, 0)
    );
    assert_eq!(
        (incoming.stored_fragments(), incoming.buffered_bytes()),
        (1, 3)
    );
    let mut output = [0; 3];
    assert_eq!(incoming.read(&mut output, 1).bytes, 3);
    assert_eq!(&output, b"new");
}
