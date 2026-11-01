// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "The helper obtains a receipt through checked public probe state."
)]
use cod4xr_client_core::handshake::{
    ChallengeBudget, ChallengeProbe, ChallengeProgress, ChallengeReceipt,
};
use std::time::Duration;

pub fn receipt(cookie: i32) -> ChallengeReceipt {
    let peer = "127.0.0.1:28960".parse().expect("Peer");
    let mut probe =
        ChallengeProbe::new(peer, 7, Duration::ZERO, ChallengeBudget::default()).expect("Probe");
    probe
        .poll(Duration::ZERO, &mut [0; 64])
        .expect("Admitted request");
    let mut packet = vec![0xff; 4];
    packet.extend_from_slice(format!("challengeResponse {cookie} 7 0 xproto 18").as_bytes());
    match probe
        .receive(peer, &packet, Duration::from_millis(1))
        .expect("Validated challenge")
    {
        ChallengeProgress::Accepted(receipt) => Some(receipt),
        ChallengeProgress::Ignored(_) => None,
    }
    .expect("The complete correlated response must be accepted")
}
