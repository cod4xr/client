// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete pending-connect output, receive, clock and phase-admission boundaries.
#![allow(
    clippy::expect_used,
    reason = "Finite boundary fixtures own valid checked request plans."
)]
mod connect_state_support;
mod connect_support;
use cod4xr_client_core::connect::{
    ConnectAction, ConnectBudget, ConnectFailure, ConnectProgress, MAX_CONNECT_PACKET_BYTES,
    NoAuthConnectAttempt,
};
use std::{net::SocketAddr, time::Duration};
fn peer() -> SocketAddr {
    "127.0.0.1:28960".parse().expect("Peer")
}
fn ms(value: u64) -> Duration {
    Duration::from_millis(value)
}

#[test]
fn every_short_output_prefix_preserves_bytes_and_phase_clock_before_a_retry() {
    let length = connect_state_support::plan().encoded_len();
    for capacity in 0..length {
        let mut state = NoAuthConnectAttempt::new(
            connect_state_support::plan(),
            ms(0),
            ConnectBudget::default(),
        )
        .expect("State");
        let usage = state.usage();
        let mut output = [0xa5; MAX_CONNECT_PACKET_BYTES];
        assert_eq!(
            state.poll(ms(500), &mut output[..capacity]),
            Err(ConnectFailure::OutputCapacity)
        );
        assert_eq!(output, [0xa5; MAX_CONNECT_PACKET_BYTES]);
        assert_eq!(state.usage(), usage);
        assert!(matches!(
            state
                .poll(ms(100), &mut output)
                .expect("Failed output did not advance time"),
            ConnectAction::Send { .. }
        ));
    }
}

#[test]
fn receive_quota_allows_its_last_acknowledgement_or_refusal_but_not_another_pending_packet() {
    for max_datagrams in 1..=12 {
        for packet in [
            b"\xff\xff\xff\xffconnectResponse".as_slice(),
            b"\xff\xff\xff\xfferror\nrefused",
            b"\xff\xff\xff\xffprint\nnotice",
            b"malformed",
        ] {
            let mut state = NoAuthConnectAttempt::new(
                connect_state_support::plan(),
                ms(0),
                ConnectBudget {
                    max_datagrams,
                    ..ConnectBudget::default()
                },
            )
            .expect("State");
            state
                .poll(ms(0), &mut [0; MAX_CONNECT_PACKET_BYTES])
                .expect("Request");
            for _ in 1..max_datagrams {
                state
                    .receive(peer(), b"noise", ms(1))
                    .expect("Remaining quota");
            }
            let result = state.receive(peer(), packet, ms(2));
            if packet.starts_with(b"\xff\xff\xff\xffconnect") {
                assert!(matches!(
                    result.expect("Last acknowledgement"),
                    ConnectProgress::Accepted(_)
                ));
            } else if packet.starts_with(b"\xff\xff\xff\xfferror") {
                assert!(matches!(
                    result.expect("Last refusal"),
                    ConnectProgress::Refused(_)
                ));
            } else {
                assert_eq!(
                    result.expect_err("Cannot remain pending past work quota"),
                    ConnectFailure::ReceiveBudget
                );
            }
            assert_eq!(state.usage().received, max_datagrams);
            assert_eq!(
                state
                    .receive(peer(), packet, ms(3))
                    .expect_err("Terminal quota"),
                ConnectFailure::Terminal
            );
        }
    }
}

#[test]
fn backwards_host_time_rejects_without_mutation_and_valid_time_recovers() {
    let mut state = NoAuthConnectAttempt::new(
        connect_state_support::plan(),
        ms(100),
        ConnectBudget::default(),
    )
    .expect("State");
    state
        .poll(ms(200), &mut [0; MAX_CONNECT_PACKET_BYTES])
        .expect("Request");
    let usage = state.usage();
    assert_eq!(
        state
            .receive(peer(), b"\xff\xff\xff\xffconnectResponse", ms(199))
            .expect_err("Clock regression"),
        ConnectFailure::TimeWentBackwards
    );
    assert_eq!(state.usage(), usage);
    assert!(matches!(
        state
            .receive(peer(), b"\xff\xff\xff\xffconnectResponse", ms(201))
            .expect("Clock recovery"),
        ConnectProgress::Accepted(_)
    ));
}

#[test]
fn zero_budgets_and_unrepresentable_deadlines_reject_without_wrapping() {
    for budget in [
        ConnectBudget {
            timeout: Duration::ZERO,
            ..ConnectBudget::default()
        },
        ConnectBudget {
            retry_interval: Duration::ZERO,
            ..ConnectBudget::default()
        },
        ConnectBudget {
            max_attempts: 0,
            ..ConnectBudget::default()
        },
        ConnectBudget {
            max_datagrams: 0,
            ..ConnectBudget::default()
        },
    ] {
        assert_eq!(
            NoAuthConnectAttempt::new(connect_state_support::plan(), ms(0), budget)
                .expect_err("Zero budget"),
            ConnectFailure::InvalidBudget
        );
    }
    assert_eq!(
        NoAuthConnectAttempt::new(
            connect_state_support::plan(),
            Duration::MAX,
            ConnectBudget::default()
        )
        .expect_err("Overflow"),
        ConnectFailure::TimeOverflow
    );
    let now = Duration::MAX
        .checked_sub(Duration::from_nanos(1))
        .expect("One tick");
    let mut state = NoAuthConnectAttempt::new(
        connect_state_support::plan(),
        now,
        ConnectBudget {
            timeout: Duration::from_nanos(1),
            retry_interval: Duration::MAX,
            max_attempts: u32::MAX,
            max_datagrams: u32::MAX,
        },
    )
    .expect("Wide bounds");
    assert!(matches!(
        state
            .poll(now, &mut [0; MAX_CONNECT_PACKET_BYTES])
            .expect("Clamped retry"),
        ConnectAction::Send {
            wake_at: Duration::MAX,
            ..
        }
    ));
    assert_eq!(
        state.poll(Duration::MAX, &mut [0; MAX_CONNECT_PACKET_BYTES]),
        Err(ConnectFailure::Expired)
    );
}
