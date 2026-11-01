// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_observability::{LogConfig, LogRuntime};
use cod4xr_client_core::QueryChallenge;
use std::process::ExitCode;

use crate::{arguments, error::ClientError, query, report};

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
            mode = "discovery",
            version = env!("CARGO_PKG_VERSION"),
            "CoD4XR client started"
        );
        let result = execute_query(&options);
        if let Err(error) = &result {
            tracing::warn!(event = "query.failed", error = %error, "Server discovery failed");
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

fn execute_query(options: &arguments::QueryOptions) -> Result<(), ClientError> {
    let mut entropy = [0; 16];
    getrandom::fill(&mut entropy).map_err(ClientError::Random)?;
    let challenge = QueryChallenge::from_entropy(entropy);
    tracing::info!(event = "query.started", peer = %options.peer, timeout_ms = options.timeout.as_millis(), "Querying server");
    let reply = query::run(options, challenge)?;
    let info = reply.info(challenge)?;
    tracing::info!(event = "query.completed", peer = %options.peer, round_trip_micros = reply.elapsed.as_micros(), received = reply.received, rejected = reply.rejected, advertised_protocol = info.advertised_protocol(), "Server discovery completed");
    report::write(options.peer, &reply, &info)
}
