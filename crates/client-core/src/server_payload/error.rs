// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

use crate::commands::ClientCommandError;
use cod4x_protocol::{compression::CompressionError, obfuscation::ObfuscationError};

/// Fixed payload-decoding failures without private commands, cookies or peer data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerPayloadError {
    /// Sequence is zero or outside the ordinary 31-bit channel domain.
    Sequence,
    /// The plaintext acknowledgement header is incomplete.
    Header,
    /// No compressed body follows the plaintext acknowledgement.
    Empty,
    /// The complete input exceeds the named baseline message budget.
    InputCapacity,
    /// Caller scratch cannot hold the complete transformed payload.
    WorkCapacity,
    /// A peer number cannot select a transmitted retained command key.
    Acknowledgement(ClientCommandError),
    /// The shared compatibility transform rejected its inputs.
    Obfuscation(ObfuscationError),
    /// Shared decompression rejected the input or destination capacity.
    Compression(CompressionError),
}

impl fmt::Display for ServerPayloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Sequence => "invalid ordinary server message sequence",
            Self::Header => "incomplete server acknowledgement header",
            Self::Empty => "server payload is empty",
            Self::InputCapacity => "server payload exceeds the baseline input budget",
            Self::WorkCapacity => "server payload scratch capacity insufficient",
            Self::Acknowledgement(_) => "server client-command acknowledgement rejected",
            Self::Obfuscation(_) => "server compatibility transform failed",
            Self::Compression(_) => "server payload decompression failed",
        })
    }
}

impl Error for ServerPayloadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Acknowledgement(error) => Some(error),
            Self::Obfuscation(error) => Some(error),
            Self::Compression(error) => Some(error),
            _ => None,
        }
    }
}
