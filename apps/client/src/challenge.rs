// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_client_core::handshake::{ChallengeBudget, ChallengeReceipt, ChallengeUsage};

use crate::{
    arguments::{RetryOptions, RuntimeOptions},
    error::ClientError,
};

pub(crate) fn run(
    options: &RuntimeOptions,
    retry: RetryOptions,
) -> (Result<ChallengeReceipt, ClientError>, ChallengeUsage) {
    let result = cod4xr_client_net::challenge(
        options.peer,
        ChallengeBudget {
            timeout: options.timeout,
            retry_interval: retry.interval,
            max_attempts: retry.attempts,
            max_datagrams: options.max_datagrams,
        },
    );
    // This is intentionally a standalone probe. The connection API retains the socket.
    (
        result
            .outcome
            .map(|socket| socket.receipt())
            .map_err(ClientError::Network),
        result.usage,
    )
}
