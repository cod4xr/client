// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{ack, complete, ms, ready, receive_storage, send_storage, wait};
use cod4xr_reliable::{
    receive::ReceiveWindow,
    schedule::{ReliableScheduler, ScheduleConfig, SendAction},
    send::{SendError, SendWindow},
};

#[test]
fn dropping_or_failing_a_callback_never_marks_data_or_advances_the_rate_cursor() {
    let (mut bytes, mut slots) = send_storage(1);
    let mut outgoing = SendWindow::new(&mut bytes, &mut slots, 1).expect("Outgoing");
    let _ = outgoing.enqueue(b"PrivateRPC").expect("Original");
    let (mut bytes, mut slots) = receive_storage(1);
    let incoming = ReceiveWindow::new(&mut bytes, &mut slots).expect("Incoming");
    let mut scheduler =
        ReliableScheduler::new(ms(0), ScheduleConfig::default()).expect("Scheduler");
    let plan = ready(&mut scheduler, ms(0), &mut outgoing, &incoming);
    assert_eq!(plan.decision().action, SendAction::Data { sequence: 0 });
    assert!(!format!("{plan:?}").contains("PrivateRPC"));
    drop(plan);
    let failed = ready(&mut scheduler, ms(0), &mut outgoing, &incoming).send_with(|packet| {
        assert_eq!(packet.payload(), b"PrivateRPC");
        Err::<usize, _>("socket failed")
    });
    assert_eq!(failed, Err("socket failed"));
    assert_eq!(
        outgoing
            .prepare_acknowledgement(ack(1, &[]))
            .expect_err("Never admitted"),
        SendError::AckUnsent
    );
    assert!(
        complete(ready(&mut scheduler, ms(0), &mut outgoing, &incoming))
            .fragment
            .expect("Actual first")
            .first_transmission
    );
    assert_eq!(
        wait(&mut scheduler, ms(249), &mut outgoing, &incoming),
        ms(250)
    );
    let sent = ready(&mut scheduler, ms(250), &mut outgoing, &incoming)
        .send_with::<()>(|packet| {
            assert_eq!(packet.header().sequence, Some(0));
            assert_eq!(packet.payload(), b"PrivateRPC");
            Ok(packet.encoded_len() + 6)
        })
        .expect("Stable retry");
    assert!(!sent.fragment.expect("Retransmission").first_transmission);
}

#[test]
fn a_wholly_selectively_acknowledged_send_span_still_allows_periodic_ack_progress() {
    let (mut bytes, mut slots) = send_storage(2);
    let mut outgoing = SendWindow::new(&mut bytes, &mut slots, 2).expect("Outgoing");
    for text in [b"first".as_slice(), b"second"] {
        let _ = outgoing.enqueue(text).expect("Original");
    }
    let (mut bytes, mut slots) = receive_storage(1);
    let incoming = ReceiveWindow::new(&mut bytes, &mut slots).expect("Incoming");
    let mut scheduler =
        ReliableScheduler::new(ms(0), ScheduleConfig::default()).expect("Scheduler");
    let _ = complete(ready(&mut scheduler, ms(0), &mut outgoing, &incoming));
    let _ = complete(ready(&mut scheduler, ms(250), &mut outgoing, &incoming));
    let _ = outgoing
        .prepare_acknowledgement(ack(0, &[(0, 2)]))
        .expect("Received both")
        .commit();
    assert_eq!(
        wait(&mut scheduler, ms(500), &mut outgoing, &incoming),
        ms(600)
    );
    let sent = complete(ready(&mut scheduler, ms(600), &mut outgoing, &incoming));
    assert_eq!(
        (sent.decision.action, sent.fragment, sent.bytes),
        (SendAction::Acknowledgement, None, 20)
    );
    assert_eq!(
        wait(&mut scheduler, ms(601), &mut outgoing, &incoming),
        ms(950)
    );
    assert_eq!(outgoing.queued_fragments(), 2);
    let _ = outgoing
        .prepare_acknowledgement(ack(2, &[]))
        .expect("Consumed")
        .commit();
    let _ = outgoing.enqueue(b"third").expect("Reuse");
    assert_eq!(
        complete(ready(&mut scheduler, ms(650), &mut outgoing, &incoming))
            .fragment
            .expect("Fresh active data")
            .sequence,
        2
    );
}
