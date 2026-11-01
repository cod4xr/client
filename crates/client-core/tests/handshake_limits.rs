// SPDX-License-Identifier: AGPL-3.0-or-later
//! Probe resource and host-clock boundaries with transactional request publication.
#![allow(
    clippy::expect_used,
    reason = "Finite boundary fixtures explicitly exercise valid and rejected budgets."
)]
use cod4xr_client_core::handshake::{
    ChallengeAction, ChallengeBudget, ChallengeFailure, ChallengeProbe, ChallengeProgress,
};
use std::{net::SocketAddr, time::Duration};

fn peer() -> SocketAddr {
    "127.0.0.1:28960".parse().expect("Peer")
}
fn ms(value: u64) -> Duration {
    Duration::from_millis(value)
}
const VALID: &[u8] = b"\xff\xff\xff\xffchallengeResponse -2147483648 1 0 xproto 18";

#[test]
fn every_short_request_prefix_leaves_bytes_clock_and_admission_unchanged() {
    let mut baseline =
        ChallengeProbe::new(peer(), i32::MIN, ms(0), ChallengeBudget::default()).expect("Baseline");
    let length = match baseline.poll(ms(0), &mut [0; 64]).expect("Request") {
        ChallengeAction::Send { length, .. } => Some(length),
        ChallengeAction::WaitUntil(_) => None,
    }
    .expect("The initial attempt must be due");
    for capacity in 0..length {
        let mut probe = ChallengeProbe::new(peer(), i32::MIN, ms(0), ChallengeBudget::default())
            .expect("Probe");
        let usage = probe.usage();
        let mut bytes = [0xa5; 64];
        assert_eq!(
            probe.poll(ms(500), &mut bytes[..capacity]),
            Err(ChallengeFailure::OutputCapacity)
        );
        assert_eq!(bytes, [0xa5; 64]);
        assert_eq!(probe.usage(), usage);
        assert!(matches!(
            probe
                .poll(ms(100), &mut bytes)
                .expect("Failed encoding did not advance clock"),
            ChallengeAction::Send { .. }
        ));
    }
}

#[test]
fn backwards_host_time_is_rejected_without_mutation_and_correct_time_recovers() {
    let mut probe =
        ChallengeProbe::new(peer(), 1, ms(100), ChallengeBudget::default()).expect("Probe");
    probe.poll(ms(200), &mut [0; 64]).expect("Request");
    let usage = probe.usage();
    assert_eq!(
        probe
            .receive(peer(), VALID, ms(199))
            .expect_err("Clock regression"),
        ChallengeFailure::TimeWentBackwards
    );
    assert_eq!(probe.usage(), usage);
    assert!(matches!(
        probe
            .receive(peer(), VALID, ms(201))
            .expect("Recovered host clock"),
        ChallengeProgress::Accepted(_)
    ));
}

#[test]
fn each_datagram_quota_accepts_its_last_valid_packet_and_terminates_on_its_last_rejection() {
    for max_datagrams in 1..=16 {
        let budget = ChallengeBudget {
            max_datagrams,
            ..ChallengeBudget::default()
        };
        for accepted in [true, false] {
            let mut probe = ChallengeProbe::new(peer(), 1, ms(0), budget).expect("Probe");
            probe.poll(ms(0), &mut [0; 64]).expect("Request");
            for _ in 1..max_datagrams {
                probe
                    .receive(peer(), b"noise", ms(1))
                    .expect("Remaining quota");
            }
            let result = probe.receive(peer(), if accepted { VALID } else { b"noise" }, ms(1));
            if accepted {
                assert!(matches!(
                    result.expect("Last valid packet"),
                    ChallengeProgress::Accepted(_)
                ));
            } else {
                assert_eq!(
                    result.expect_err("Last malformed packet"),
                    ChallengeFailure::ReceiveBudget
                );
            }
            assert_eq!(probe.usage().received, max_datagrams);
            assert_eq!(
                probe.receive(peer(), VALID, ms(2)).expect_err("Terminal"),
                ChallengeFailure::Terminal
            );
            assert_eq!(probe.usage().received, max_datagrams);
        }
    }
}

#[test]
fn zero_budgets_and_deadline_overflow_reject_and_retry_overflow_clamps_to_expiry() {
    for budget in [
        ChallengeBudget {
            timeout: Duration::ZERO,
            ..ChallengeBudget::default()
        },
        ChallengeBudget {
            retry_interval: Duration::ZERO,
            ..ChallengeBudget::default()
        },
        ChallengeBudget {
            max_attempts: 0,
            ..ChallengeBudget::default()
        },
        ChallengeBudget {
            max_datagrams: 0,
            ..ChallengeBudget::default()
        },
    ] {
        assert_eq!(
            ChallengeProbe::new(peer(), 1, ms(0), budget).expect_err("Zero budget"),
            ChallengeFailure::InvalidBudget
        );
    }
    assert_eq!(
        ChallengeProbe::new(peer(), 1, Duration::MAX, ChallengeBudget::default())
            .expect_err("Deadline overflow"),
        ChallengeFailure::TimeOverflow
    );
    let started = Duration::MAX
        .checked_sub(Duration::from_nanos(1))
        .expect("One representable tick");
    let mut probe = ChallengeProbe::new(
        peer(),
        1,
        started,
        ChallengeBudget {
            timeout: Duration::from_nanos(1),
            retry_interval: Duration::MAX,
            max_attempts: u32::MAX,
            max_datagrams: u32::MAX,
        },
    )
    .expect("Wide clock and quotas");
    assert!(matches!(
        probe.poll(started, &mut [0; 64]).expect("Clamped retry"),
        ChallengeAction::Send {
            wake_at: Duration::MAX,
            ..
        }
    ));
    assert_eq!(
        probe.poll(Duration::MAX, &mut [0; 64]),
        Err(ChallengeFailure::Expired)
    );
}
