// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt, io};

#[derive(Debug)]
pub(crate) enum ClientError {
    Arguments(&'static str),
    Io(io::Error),
    Random(getrandom::Error),
    Logging(cod4x_observability::LogError),
    Output(serde_json::Error),
    Timeout,
    ReceiveBudget,
    InvalidResponse,
    Allocation,
}

impl fmt::Display for ClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arguments(message) => formatter.write_str(message),
            Self::Io(error) => write!(formatter, "network/output I/O failed: {error}"),
            Self::Random(_) => formatter.write_str("OS query randomness unavailable"),
            Self::Logging(error) => write!(formatter, "logging setup failed: {error}"),
            Self::Output(error) => write!(formatter, "report output failed: {error}"),
            Self::Timeout => formatter.write_str("server query timed out"),
            Self::ReceiveBudget => formatter.write_str("query receive budget exhausted"),
            Self::InvalidResponse => formatter.write_str("invalid checked response state"),
            Self::Allocation => formatter.write_str("query receive buffer allocation failed"),
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Random(error) => Some(error),
            Self::Logging(error) => Some(error),
            Self::Output(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for ClientError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
