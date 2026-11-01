// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{net::SocketAddr, time::Duration};

use cod4x_observability::ConsoleFormat;

use crate::error::ClientError;

pub(crate) const HELP: &str = "CoD4XR client development runtime\n\
Connectionless probes only; gameplay/rendering are not implemented. No-auth baseline.\n\n\
Usage: cod4xr-client query IP:PORT [--timeout-ms 2000] [--max-datagrams 64]\n\
                         [--log-format human|json]\n\
       cod4xr-client probe-challenge IP:PORT [common options]\n\
                         [--retry-ms 500] [--max-attempts 3]\n\
       cod4xr-client --help\n\n\
IPv6 endpoints use [ADDRESS]:PORT. Reports are JSON; logs use stderr.\n\
The challenge probe accepts only the 0 xproto 18 profile and attempts no join.\n";

#[derive(Clone, Copy)]
pub(crate) struct RetryOptions {
    pub(crate) interval: Duration,
    pub(crate) attempts: u32,
}

#[derive(Clone, Copy)]
pub(crate) enum CommandKind {
    Query,
    Challenge(RetryOptions),
}

impl CommandKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Query => "discovery",
            Self::Challenge(_) => "challenge_probe",
        }
    }
}

pub(crate) struct RuntimeOptions {
    pub(crate) command: CommandKind,
    pub(crate) peer: SocketAddr,
    pub(crate) timeout: Duration,
    pub(crate) max_datagrams: u32,
    pub(crate) logs: ConsoleFormat,
}

pub(crate) fn from_environment() -> Result<Option<RuntimeOptions>, ClientError> {
    // There are at most twelve implemented argument/value positions. Keep OS
    // argument conversion bounded and reject non-Unicode input without panics.
    let values = std::env::args_os()
        .skip(1)
        .take(13)
        .map(|value| {
            value
                .into_string()
                .map_err(|_| ClientError::Arguments("arguments must be valid Unicode"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() > 12 {
        return Err(ClientError::Arguments("too many command arguments"));
    }
    parse(values.into_iter())
}

pub(crate) fn parse(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<RuntimeOptions>, ClientError> {
    let Some(command) = args.next() else {
        return Ok(None);
    };
    if matches!(command.as_str(), "--help" | "-h") {
        if args.next().is_some() {
            return Err(ClientError::Arguments("unexpected help arguments"));
        }
        return Ok(None);
    }
    let command = match command.as_str() {
        "query" => CommandKind::Query,
        "probe-challenge" => CommandKind::Challenge(RetryOptions {
            interval: Duration::from_millis(500),
            attempts: 3,
        }),
        _ => {
            return Err(ClientError::Arguments(
                "expected query, probe-challenge or --help",
            ));
        }
    };
    let peer = args
        .next()
        .and_then(|value| value.parse::<SocketAddr>().ok())
        .filter(|peer| peer.port() != 0 && !peer.ip().is_unspecified() && !peer.ip().is_multicast())
        .ok_or(ClientError::Arguments(
            "expected a unicast IP:PORT with a nonzero port",
        ))?;
    let mut options = RuntimeOptions {
        command,
        peer,
        timeout: Duration::from_secs(2),
        max_datagrams: 64,
        logs: ConsoleFormat::Auto,
    };
    let mut seen = [false; 5];
    while let Some(option) = args.next() {
        let index = match option.as_str() {
            "--timeout-ms" => 0,
            "--max-datagrams" => 1,
            "--log-format" => 2,
            "--retry-ms" => 3,
            "--max-attempts" => 4,
            _ => return Err(ClientError::Arguments("unknown query option")),
        };
        if seen[index] {
            return Err(ClientError::Arguments("duplicate query option"));
        }
        seen[index] = true;
        let value = args
            .next()
            .ok_or(ClientError::Arguments("missing query option value"))?;
        match index {
            0 => options.timeout = Duration::from_millis(number(&value, 30_000)?),
            1 => {
                options.max_datagrams = u32::try_from(number(&value, 4096)?)
                    .map_err(|_| ClientError::Arguments("invalid datagram budget"))?;
            }
            2 => {
                options.logs = match value.as_str() {
                    "human" => ConsoleFormat::Human,
                    "json" => ConsoleFormat::Json,
                    _ => return Err(ClientError::Arguments("expected human or json log format")),
                }
            }
            _ => {
                let CommandKind::Challenge(retry) = &mut options.command else {
                    return Err(ClientError::Arguments(
                        "retry options require probe-challenge",
                    ));
                };
                if index == 3 {
                    retry.interval = Duration::from_millis(number(&value, 30_000)?);
                } else {
                    retry.attempts = u32::try_from(number(&value, 16)?)
                        .map_err(|_| ClientError::Arguments("invalid attempt quota"))?;
                }
            }
        }
    }
    Ok(Some(options))
}

fn number(value: &str, maximum: u64) -> Result<u64, ClientError> {
    value
        .parse::<u64>()
        .ok()
        .filter(|value| (1..=maximum).contains(value))
        .ok_or(ClientError::Arguments(
            "query limit is outside its allowed range",
        ))
}
