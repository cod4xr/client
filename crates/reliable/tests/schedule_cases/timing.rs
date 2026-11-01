// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Duration;

use crate::{complete, ms, ready, receive_storage, send_storage, wait};
use cod4xr_reliable::{
    receive::ReceiveWindow,
    schedule::{ReliableScheduler, ScheduleConfig, ScheduleError, SendAction},
    send::SendWindow,
};

#[test]
fn configuration_rejects_zero_or_inconsistent_intervals_and_initial_timer_overflow() {
    let base = ScheduleConfig::default();
    assert_eq!(
        (
            base.minimum_interval,
            base.data_interval,
            base.acknowledgement_interval
        ),
        (ms(20), ms(250), ms(350))
    );
    for config in [
        ScheduleConfig {
            minimum_interval: Duration::ZERO,
            ..base
        },
        ScheduleConfig {
            data_interval: Duration::ZERO,
            ..base
        },
        ScheduleConfig {
            acknowledgement_interval: Duration::ZERO,
            ..base
        },
        ScheduleConfig {
            minimum_interval: ms(251),
            ..base
        },
        ScheduleConfig {
            acknowledgement_interval: ms(19),
            ..base
        },
    ] {
        assert_eq!(
            ReliableScheduler::new(Duration::ZERO, config).expect_err("Timing policy"),
            ScheduleError::Configuration
        );
    }
    assert_eq!(
        ReliableScheduler::new(Duration::MAX, base).expect_err("Initial overflow"),
        ScheduleError::ClockOverflow
    );
}

#[test]
fn initial_data_is_immediate_round_robin_is_paced_and_data_headers_refresh_the_ack_timer() {
    let (mut bytes, mut slots) = send_storage(4);
    let mut outgoing = SendWindow::new(&mut bytes, &mut slots, 4).expect("Outgoing");
    let (mut bytes, mut slots) = receive_storage(1);
    let incoming = ReceiveWindow::new(&mut bytes, &mut slots).expect("Incoming");
    let mut scheduler =
        ReliableScheduler::new(ms(0), ScheduleConfig::default()).expect("Scheduler");
    assert_eq!(
        wait(&mut scheduler, ms(0), &mut outgoing, &incoming),
        ms(350)
    );
    for text in [b"first".as_slice(), b"second", b"third"] {
        let _ = outgoing.enqueue(text).expect("Original data");
    }
    for (tick, sequence) in [(0, 0), (250, 1), (500, 2), (750, 0)] {
        let plan = ready(&mut scheduler, ms(tick), &mut outgoing, &incoming);
        assert_eq!(plan.decision().action, SendAction::Data { sequence });
        let sent = complete(plan);
        assert_eq!(sent.fragment.expect("Data").first_transmission, tick != 750);
        assert_eq!(
            wait(&mut scheduler, ms(tick), &mut outgoing, &incoming),
            ms(tick + 250)
        );
    }
    assert_eq!(
        wait(&mut scheduler, ms(999), &mut outgoing, &incoming),
        ms(1000)
    );
    assert_eq!(outgoing.acknowledge(), 0);
}

#[test]
fn a_long_pause_admits_one_datagram_without_accumulated_credits_or_a_catch_up_burst() {
    let (mut bytes, mut slots) = send_storage(4);
    let mut outgoing = SendWindow::new(&mut bytes, &mut slots, 4).expect("Outgoing");
    let _ = outgoing.enqueue(&[0; 2400]).expect("Two fragments");
    let (mut bytes, mut slots) = receive_storage(1);
    let incoming = ReceiveWindow::new(&mut bytes, &mut slots).expect("Incoming");
    let mut scheduler =
        ReliableScheduler::new(ms(0), ScheduleConfig::default()).expect("Scheduler");
    let sent = complete(ready(&mut scheduler, ms(100_000), &mut outgoing, &incoming));
    assert_eq!(sent.fragment.expect("One fragment").sequence, 0);
    for _ in 0..100 {
        assert_eq!(
            wait(&mut scheduler, ms(100_000), &mut outgoing, &incoming),
            ms(100_250)
        );
    }
    assert_eq!(
        complete(ready(&mut scheduler, ms(100_250), &mut outgoing, &incoming))
            .fragment
            .expect("Next fragment")
            .sequence,
        1
    );
}
