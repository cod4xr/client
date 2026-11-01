// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pending connect state binds acknowledgements to admitted requests and exact peers.
#![allow(
    clippy::expect_used,
    reason = "Finite explicit clock/reply fixtures have valid phase budgets."
)]
mod connect_state_support;
mod connect_support;
use cod4xr_client_core::connect::{
    ConnectAction, ConnectBudget, ConnectFailure, ConnectProgress, ConnectRejection,
    ConnectReplyKind, MAX_CONNECT_PACKET_BYTES, NoAuthConnectAttempt,
};
use std::{net::SocketAddr, time::Duration};

fn peer() -> SocketAddr {
    "127.0.0.1:28960".parse().expect("Peer")
}
fn ms(value: u64) -> Duration {
    Duration::from_millis(value)
}
fn attempt() -> NoAuthConnectAttempt {
    NoAuthConnectAttempt::new(
        connect_state_support::plan(),
        ms(0),
        ConnectBudget::default(),
    )
    .expect("Pending phase")
}

#[test]
fn unsolicited_and_wrong_endpoint_packets_cannot_admit_a_connection() {
    let mut state = attempt();
    let packet = b"\xff\xff\xff\xffconnectResponse mods/PrivateMod";
    assert!(matches!(
        state.receive(peer(), packet, ms(0)).expect("Unsolicited"),
        ConnectProgress::Ignored(ConnectRejection::Unsolicited)
    ));
    state
        .poll(ms(1), &mut [0; MAX_CONNECT_PACKET_BYTES])
        .expect("Initial request");
    for address in ["127.0.0.2:28960", "127.0.0.1:28961"] {
        assert!(matches!(
            state
                .receive(address.parse().expect("Wrong peer"), packet, ms(2))
                .expect("Ignored"),
            ConnectProgress::Ignored(ConnectRejection::WrongPeer)
        ));
    }
    let observation = match state
        .receive(peer(), packet, ms(9))
        .expect("Checked acknowledgement")
    {
        ConnectProgress::Accepted(observation) => Some(observation),
        _ => None,
    }
    .expect("The selected pending peer must be accepted");
    assert_eq!(observation.peer(), peer());
    assert_eq!(observation.requested_protocol(), 21);
    assert_eq!(observation.qport(), 40_000);
    assert_eq!(observation.kind(), ConnectReplyKind::Accepted);
    assert_eq!(observation.payload(), b"mods/PrivateMod");
    assert_eq!(observation.elapsed(), ms(9));
    assert_eq!(
        (
            observation.usage().attempts,
            observation.usage().received,
            observation.usage().rejected
        ),
        (1, 4, 3)
    );
    for debug in [format!("{state:?}"), format!("{observation:?}")] {
        for private in [
            "PrivatePlayer",
            "PrivatePassword",
            "PrivateMod",
            "2147483648",
            "127.0.0.1",
        ] {
            assert!(!debug.contains(private));
        }
    }
    assert_eq!(
        state
            .receive(peer(), packet, ms(10))
            .expect_err("One acknowledgement only"),
        ConnectFailure::Terminal
    );
}

#[test]
fn retries_reuse_identical_owned_bytes_and_accept_after_the_last_attempt() {
    let mut state = attempt();
    let mut first = [0; MAX_CONNECT_PACKET_BYTES];
    let length = match state.poll(ms(0), &mut first).expect("First request") {
        ConnectAction::Send { length, .. } => Some(length),
        ConnectAction::WaitUntil(_) => None,
    }
    .expect("Initial request is due");
    assert!(first[..length].starts_with(b"\xff\xff\xff\xffconnect \""));
    for (now, next) in [(500, 1000), (1000, 3000)] {
        let mut output = [0; MAX_CONNECT_PACKET_BYTES];
        assert_eq!(
            state.poll(ms(now), &mut output),
            Ok(ConnectAction::Send {
                length,
                wake_at: ms(next)
            })
        );
        assert_eq!(output, first);
    }
    assert_eq!(
        state.poll(ms(1500), &mut first),
        Ok(ConnectAction::WaitUntil(ms(3000)))
    );
    assert!(matches!(
        state
            .receive(peer(), b"\xff\xff\xff\xffconnectResponse", ms(2999))
            .expect("Late eligible reply"),
        ConnectProgress::Accepted(_)
    ));
    assert_eq!(state.usage().attempts, 3);
}

#[test]
fn notices_remain_data_and_refusal_is_distinct_from_admission_or_auth_execution() {
    let mut state = attempt();
    state
        .poll(ms(0), &mut [0; MAX_CONNECT_PACKET_BYTES])
        .expect("Request");
    let notice = match state
        .receive(peer(), b"\xff\xff\xff\xffprint\nPrivateMessage", ms(10))
        .expect("Notice")
    {
        ConnectProgress::Notice(observation) => Some(observation),
        _ => None,
    }
    .expect("Print must remain informational");
    assert_eq!(notice.payload(), b"PrivateMessage");
    assert_eq!(notice.kind(), ConnectReplyKind::Notice);
    assert_eq!(state.usage().notices, 1);
    let refusal = match state
        .receive(
            peer(),
            b"\xff\xff\xff\xfferror\n{ OOBErrorParser steamonly PrivateReason }",
            ms(20),
        )
        .expect("Refusal")
    {
        ConnectProgress::Refused(observation) => Some(observation),
        _ => None,
    }
    .expect("Error must refuse instead of admitting");
    assert!(refusal.payload().starts_with(b"{ OOBErrorParser"));
    assert_eq!(refusal.kind(), ConnectReplyKind::Refused);
    assert_eq!(refusal.usage().rejected, 0);
    assert_eq!(refusal.usage().received, 2);
    assert!(!format!("{refusal:?}").contains("PrivateReason"));
    assert_eq!(
        state.poll(ms(21), &mut [0; MAX_CONNECT_PACKET_BYTES]),
        Err(ConnectFailure::Terminal)
    );
}

#[test]
fn valid_notices_and_rejected_traffic_do_not_extend_the_absolute_deadline() {
    let mut state = attempt();
    state
        .poll(ms(0), &mut [0; MAX_CONNECT_PACKET_BYTES])
        .expect("Request");
    assert!(matches!(
        state
            .receive(peer(), b"\xff\xff\xff\xffprint\nwaiting", ms(2998))
            .expect("Notice"),
        ConnectProgress::Notice(_)
    ));
    assert!(matches!(
        state
            .receive(peer(), b"malformed", ms(2999))
            .expect("Noise"),
        ConnectProgress::Ignored(ConnectRejection::Malformed)
    ));
    let usage = state.usage();
    assert_eq!(
        state
            .receive(peer(), b"\xff\xff\xff\xffconnectResponse", ms(3000))
            .expect_err("Absolute expiry"),
        ConnectFailure::Expired
    );
    assert_eq!(state.usage(), usage);
    assert_eq!(
        state.poll(ms(3001), &mut [0; MAX_CONNECT_PACKET_BYTES]),
        Err(ConnectFailure::Terminal)
    );
}
