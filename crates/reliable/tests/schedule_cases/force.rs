// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Duration;

use crate::{complete, ms, ready, receive_storage, send_storage, wait};
use cod4xr_reliable::{
    receive::ReceiveWindow,
    schedule::{ReliableScheduler, ScheduleConfig, ScheduleError, SendAction},
    send::SendWindow,
};

#[test]
fn repeated_validated_force_acks_cannot_bypass_the_global_interval_and_data_can_carry_the_ack() {
    let (mut bytes, mut slots) = send_storage(1);
    let mut outgoing = SendWindow::new(&mut bytes, &mut slots, 1).expect("Outgoing");
    let (mut bytes, mut slots) = receive_storage(1);
    let incoming = ReceiveWindow::new(&mut bytes, &mut slots).expect("Incoming");
    let mut scheduler =
        ReliableScheduler::new(ms(0), ScheduleConfig::default()).expect("Scheduler");
    scheduler
        .observe_validated_packet(ms(1), true)
        .expect("Validated force");
    assert_eq!(
        complete(ready(&mut scheduler, ms(1), &mut outgoing, &incoming))
            .decision
            .action,
        SendAction::Acknowledgement
    );
    for _ in 0..100 {
        scheduler
            .observe_validated_packet(ms(1), true)
            .expect("Same-tick valid input");
    }
    assert_eq!(
        wait(&mut scheduler, ms(1), &mut outgoing, &incoming),
        ms(21)
    );
    assert_eq!(
        wait(&mut scheduler, ms(20), &mut outgoing, &incoming),
        ms(21)
    );
    let _ = complete(ready(&mut scheduler, ms(21), &mut outgoing, &incoming));
    let _ = outgoing.enqueue(b"data").expect("New data");
    scheduler
        .observe_validated_packet(ms(22), true)
        .expect("Validated force");
    assert_eq!(
        wait(&mut scheduler, ms(22), &mut outgoing, &incoming),
        ms(41)
    );
    let sent = complete(ready(&mut scheduler, ms(41), &mut outgoing, &incoming));
    assert_eq!(sent.decision.action, SendAction::Data { sequence: 0 });
    assert_eq!(
        wait(&mut scheduler, ms(41), &mut outgoing, &incoming),
        ms(291)
    );
}

#[test]
fn rejected_clocks_cannot_mutate_force_eligibility_or_poison_the_last_admitted_time() {
    let (mut bytes, mut slots) = send_storage(1);
    let mut outgoing = SendWindow::new(&mut bytes, &mut slots, 1).expect("Outgoing");
    let (mut bytes, mut slots) = receive_storage(1);
    let incoming = ReceiveWindow::new(&mut bytes, &mut slots).expect("Incoming");
    let mut scheduler =
        ReliableScheduler::new(ms(0), ScheduleConfig::default()).expect("Scheduler");
    assert_eq!(
        wait(&mut scheduler, ms(100), &mut outgoing, &incoming),
        ms(350)
    );
    assert_eq!(
        scheduler.observe_validated_packet(ms(99), true),
        Err(ScheduleError::ClockRegressed)
    );
    assert_eq!(
        scheduler
            .poll(ms(99), &mut outgoing, &incoming)
            .expect_err("Backwards"),
        ScheduleError::ClockRegressed
    );
    assert_eq!(
        wait(&mut scheduler, ms(100), &mut outgoing, &incoming),
        ms(350)
    );
    assert_eq!(
        scheduler
            .poll(Duration::MAX, &mut outgoing, &incoming)
            .expect_err("Future timer overflow"),
        ScheduleError::ClockOverflow
    );
    assert_eq!(
        wait(&mut scheduler, ms(101), &mut outgoing, &incoming),
        ms(350)
    );
    let _ = outgoing.enqueue(b"data").expect("Still usable");
    assert_eq!(
        complete(ready(&mut scheduler, ms(101), &mut outgoing, &incoming))
            .fragment
            .expect("Data")
            .sequence,
        0
    );
}
