// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::{Duration, Instant};

use cod4xr_client_core::handshake::{
    ChallengeAction, ChallengeBudget, ChallengeFailure, ChallengeProbe, ChallengeProgress,
    ChallengeReceipt, ChallengeUsage,
};

use crate::{
    arguments::{RetryOptions, RuntimeOptions},
    error::ClientError,
    net_io,
};

pub(crate) fn run(
    options: &RuntimeOptions,
    retry: RetryOptions,
    nonce: i32,
) -> (Result<ChallengeReceipt, ClientError>, ChallengeUsage) {
    let started = Instant::now();
    let budget = ChallengeBudget {
        timeout: options.timeout,
        retry_interval: retry.interval,
        max_attempts: retry.attempts,
        max_datagrams: options.max_datagrams,
    };
    let mut probe = match ChallengeProbe::new(options.peer, nonce, Duration::ZERO, budget) {
        Ok(probe) => probe,
        Err(error) => {
            return (
                Err(ClientError::Handshake(error)),
                ChallengeUsage::default(),
            );
        }
    };
    let result = pending(options, &mut probe, started);
    (result, probe.usage())
}

fn pending(
    options: &RuntimeOptions,
    probe: &mut ChallengeProbe,
    started: Instant,
) -> Result<ChallengeReceipt, ClientError> {
    let socket = net_io::connect(options.peer)?;
    let deadline = started
        .checked_add(options.timeout)
        .ok_or(ClientError::Timeout)?;
    let mut incoming = net_io::receive_buffer()?;
    let mut request = [0; 64];
    loop {
        let action = probe
            .poll(started.elapsed(), &mut request)
            .map_err(ClientError::Handshake)?;
        let wake_at = match action {
            ChallengeAction::Send { length, wake_at } => {
                net_io::send(&socket, &request[..length], deadline)?;
                wake_at
            }
            ChallengeAction::WaitUntil(wake_at) => wake_at,
        };
        let wake_at = started
            .checked_add(wake_at)
            .ok_or(ClientError::Handshake(ChallengeFailure::TimeOverflow))?;
        if let Some(length) = net_io::receive_until(&socket, &mut incoming, wake_at)? {
            match probe
                .receive(options.peer, &incoming[..length], started.elapsed())
                .map_err(ClientError::Handshake)?
            {
                ChallengeProgress::Ignored(_) => {}
                ChallengeProgress::Accepted(receipt) => return Ok(receipt),
            }
        }
    }
}
