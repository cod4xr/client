// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{complete, ms, ready, receive_slot, receive_storage, send_storage};
use cod4xr_reliable::{
    SelectiveAckRange,
    receive::ReceiveWindow,
    schedule::{ReliableScheduler, ScheduleConfig},
    send::SendWindow,
};

#[test]
fn successful_ack_pages_rotate_past_three_ranges_and_reset_when_the_consumed_base_changes() {
    let (mut bytes, mut slots) = send_storage(4);
    let mut outgoing = SendWindow::new(&mut bytes, &mut slots, 4).expect("Outgoing");
    let (mut bytes, mut slots) = receive_storage(20);
    let mut incoming = ReceiveWindow::new(&mut bytes, &mut slots).expect("Incoming");
    for sequence in [1, 3, 5, 7, 9, 11] {
        receive_slot(&mut incoming, sequence);
    }
    let mut scheduler =
        ReliableScheduler::new(ms(0), ScheduleConfig::default()).expect("Scheduler");
    let first = complete(ready(&mut scheduler, ms(350), &mut outgoing, &incoming));
    let ranges = [1, 3, 5].map(|offset| SelectiveAckRange { offset, length: 1 });
    assert_eq!(first.decision.receive.selective.ranges(), ranges);
    assert_eq!(first.decision.receive.next_offset, 6);
    receive_slot(&mut incoming, 0);
    assert_eq!(incoming.read(&mut [0; 1], 2).fragments, 2);
    assert_eq!(incoming.acknowledge(), 2);
    let second = complete(ready(&mut scheduler, ms(700), &mut outgoing, &incoming));
    assert_eq!(second.decision.receive.cumulative, 2);
    assert_eq!(second.decision.receive.selective.ranges(), ranges);
    assert_eq!(second.decision.receive.next_offset, 6);
    let third = complete(ready(&mut scheduler, ms(1050), &mut outgoing, &incoming));
    let expected = [7, 9].map(|offset| SelectiveAckRange { offset, length: 1 });
    assert_eq!(third.decision.receive.selective.ranges(), expected);
    assert_eq!(third.decision.receive.next_offset, 0);
    assert_eq!((first.bytes, second.bytes, third.bytes), (32, 32, 28));
    assert_eq!(
        (
            outgoing.acknowledge(),
            outgoing.next_sequence(),
            outgoing.queued_fragments()
        ),
        (0, 0, 0)
    );
}
