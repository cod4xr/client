// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_observability::{LogConfig, LogRuntime};
use cod4xr_client_core::QueryChallenge;
use std::process::ExitCode;

use crate::{
    arguments::{self, CommandKind, RetryOptions, RuntimeOptions},
    challenge, challenge_report,
    error::ClientError,
    query, report,
};

pub(crate) fn run() -> Result<ExitCode, ClientError> {
    let Some(options) = arguments::from_environment()? else {
        print!("{}", arguments::HELP);
        return Ok(ExitCode::SUCCESS);
    };
    let mut config = LogConfig::default();
    config.console_format = options.logs;
    let logs = LogRuntime::build(&config).map_err(ClientError::Logging)?;
    let code = tracing::dispatcher::with_default(logs.dispatch(), || {
        tracing::info!(
            event = "client.started",
            project = "CoD4XR",
            mode = options.command.label(),
            version = env!("CARGO_PKG_VERSION"),
            "CoD4XR client started"
        );
        let result = match options.command {
            CommandKind::Query => execute_query(&options),
            CommandKind::Challenge(retry) => execute_challenge(&options, retry),
        };
        if let Err(error) = &result {
            let event = match options.command {
                CommandKind::Query => "query.failed",
                CommandKind::Challenge(_) => "challenge.failed",
            };
            tracing::warn!(event, error = %error, "Client operation failed");
        }
        tracing::info!(
            event = "client.stopped",
            dropped_logs = logs.dropped_records(),
            "CoD4XR client stopped"
        );
        if result.is_ok() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    });
    Ok(code)
}

fn execute_query(options: &RuntimeOptions) -> Result<(), ClientError> {
    let mut entropy = [0; 16];
    getrandom::fill(&mut entropy).map_err(ClientError::Random)?;
    let challenge = QueryChallenge::from_entropy(entropy);
    tracing::info!(event = "query.started", peer = %options.peer, timeout_ms = options.timeout.as_millis(), "Querying server");
    let reply = query::run(options, challenge)?;
    let info = reply.info(challenge)?;
    tracing::info!(event = "query.completed", peer = %options.peer, round_trip_micros = reply.elapsed.as_micros(), received = reply.received, rejected = reply.rejected, advertised_protocol = info.advertised_protocol(), "Server discovery completed");
    report::write(options.peer, &reply, &info)
}

fn execute_challenge(options: &RuntimeOptions, retry: RetryOptions) -> Result<(), ClientError> {
    let mut entropy = [0; 4];
    getrandom::fill(&mut entropy).map_err(ClientError::Random)?;
    let nonce = i32::from_le_bytes(entropy);
    tracing::info!(event = "challenge.started", peer = %options.peer, timeout_ms = options.timeout.as_millis(), max_attempts = retry.attempts, "Probing server challenge");
    let (result, usage) = challenge::run(options, retry, nonce);
    tracing::info!(event = "challenge.summary", attempts = usage.attempts, received = usage.received, rejected = usage.rejected, last_rejection = ?usage.last_rejection, "Challenge work summary");
    let receipt = result?;
    tracing::info!(
        event = "challenge.accepted",
        elapsed_micros = receipt.elapsed().as_micros(),
        "Return-path challenge accepted"
    );
    challenge_report::write(&receipt)
}
