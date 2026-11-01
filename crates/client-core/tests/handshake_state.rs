// SPDX-License-Identifier: AGPL-3.0-or-later
//! Challenge transitions, correlation, redaction, retransmission and terminal behavior.
#![allow(
    clippy::expect_used,
    reason = "Finite explicit clock/packet fixtures have valid budgets."
)]
use cod4xr_client_core::handshake::{
    ChallengeAction, ChallengeBudget, ChallengeFailure, ChallengeProbe, ChallengeProgress,
    ChallengeRejection,
};
use std::{net::SocketAddr, time::Duration};

fn peer() -> SocketAddr {
    "127.0.0.1:28960".parse().expect("Selected peer")
}
fn ms(time: u64) -> Duration {
    Duration::from_millis(time)
}
fn response(cookie: i32, nonce: i32) -> Vec<u8> {
    let mut bytes = vec![0xff; 4];
    bytes.extend_from_slice(format!("challengeResponse {cookie} {nonce} 0 xproto 18").as_bytes());
    bytes
}
fn start(nonce: i32) -> ChallengeProbe {
    ChallengeProbe::new(peer(), nonce, ms(0), ChallengeBudget::default()).expect("Probe")
}

#[test]
fn accepted_cookie_requires_a_pending_request_and_is_published_once_without_debug_disclosure() {
    let mut probe = start(i32::MIN);
    let packet = response(i32::MAX, i32::MIN);
    assert!(matches!(
        probe.receive(peer(), &packet, ms(0)).expect("Unsolicited"),
        ChallengeProgress::Ignored(ChallengeRejection::Unsolicited)
    ));
    let mut output = [0; 64];
    assert!(matches!(
        probe.poll(ms(1), &mut output).expect("Initial request"),
        ChallengeAction::Send { .. }
    ));
    let receipt = match probe
        .receive(peer(), &packet, ms(9))
        .expect("Checked reply")
    {
        ChallengeProgress::Accepted(receipt) => Some(receipt),
        ChallengeProgress::Ignored(_) => None,
    }
    .expect("A valid pending reply must be accepted");
    assert_eq!(receipt.server_challenge(), i32::MAX);
    assert_eq!(receipt.peer(), peer());
    assert_eq!(receipt.elapsed(), ms(9));
    assert_eq!(
        (
            receipt.usage().attempts,
            receipt.usage().received,
            receipt.usage().rejected
        ),
        (1, 2, 1)
    );
    for debug in [format!("{probe:?}"), format!("{receipt:?}")] {
        assert!(!debug.contains("2147483647"));
        assert!(!debug.contains("-2147483648"));
        assert!(!debug.contains("127.0.0.1"));
    }
    assert_eq!(
        probe
            .receive(peer(), &packet, ms(10))
            .expect_err("No repeated receipt"),
        ChallengeFailure::Terminal
    );
    assert_eq!(
        probe.poll(ms(10), &mut output),
        Err(ChallengeFailure::Terminal)
    );
}

#[test]
fn exact_peer_and_nonce_checks_precede_state_advancement() {
    let mut probe = start(123);
    probe.poll(ms(0), &mut [0; 64]).expect("Request");
    for (address, packet, reason) in [
        (
            "127.0.0.1:28961",
            b"malformed".to_vec(),
            ChallengeRejection::WrongPeer,
        ),
        (
            "127.0.0.2:28960",
            response(5, 123),
            ChallengeRejection::WrongPeer,
        ),
        (
            "127.0.0.1:28960",
            response(5, 124),
            ChallengeRejection::WrongNonce,
        ),
        (
            "127.0.0.1:28960",
            b"\xff\xff\xff\xffchallengeResponse 5 123 0 xproto 21".to_vec(),
            ChallengeRejection::UnsupportedProfile,
        ),
    ] {
        assert!(
            matches!(probe.receive(address.parse().expect("Peer"), &packet, ms(1)).expect("Ignored"),
            ChallengeProgress::Ignored(found) if found == reason)
        );
    }
    assert_eq!((probe.usage().received, probe.usage().rejected), (4, 4));
    assert!(matches!(
        probe
            .receive(peer(), &response(5, 123), ms(2))
            .expect("Correct peer/nonce"),
        ChallengeProgress::Accepted(_)
    ));
}

#[test]
fn retries_keep_the_nonce_and_absolute_deadline_and_allow_a_late_reply_after_last_attempt() {
    let mut probe = start(-7);
    let mut initial = [0; 64];
    let (length, wake_at) = match probe.poll(ms(0), &mut initial).expect("First") {
        ChallengeAction::Send { length, wake_at } => Some((length, wake_at)),
        ChallengeAction::WaitUntil(_) => None,
    }
    .expect("The initial attempt must be due");
    assert_eq!(wake_at, ms(500));
    assert_eq!(&initial[..length], b"\xff\xff\xff\xffgetchallenge -7");
    assert_eq!(
        probe.poll(ms(499), &mut [0; 64]),
        Ok(ChallengeAction::WaitUntil(ms(500)))
    );
    for (time, next) in [(500, 1000), (1000, 2000)] {
        let mut request = [0; 64];
        assert_eq!(
            probe.poll(ms(time), &mut request),
            Ok(ChallengeAction::Send {
                length,
                wake_at: ms(next)
            })
        );
        assert_eq!(&request[..length], &initial[..length]);
    }
    assert_eq!(
        probe.poll(ms(1100), &mut [0; 64]),
        Ok(ChallengeAction::WaitUntil(ms(2000)))
    );
    assert!(matches!(
        probe
            .receive(peer(), &response(0, -7), ms(1900))
            .expect("Last-attempt response"),
        ChallengeProgress::Accepted(_)
    ));
    assert_eq!(probe.usage().attempts, 3);
}

#[test]
fn rejected_traffic_does_not_extend_expiry_or_allow_a_response_on_the_deadline() {
    let mut probe = start(1);
    probe.poll(ms(0), &mut [0; 64]).expect("Request");
    for time in [100, 500, 1000, 1999] {
        assert!(matches!(
            probe
                .receive(peer(), b"noise", ms(time))
                .expect("Ignored noise"),
            ChallengeProgress::Ignored(ChallengeRejection::Malformed)
        ));
    }
    let usage = probe.usage();
    assert_eq!(
        probe
            .receive(peer(), &response(9, 1), ms(2000))
            .expect_err("Absolute expiry"),
        ChallengeFailure::Expired
    );
    assert_eq!(probe.usage(), usage);
    assert_eq!(
        probe.poll(ms(2001), &mut [0; 64]),
        Err(ChallengeFailure::Terminal)
    );
}
