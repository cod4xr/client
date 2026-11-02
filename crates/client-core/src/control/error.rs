// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

use cod4x_protocol::{MessageError, compression::CompressionError, obfuscation::ObfuscationError};

use super::MovementMessageError;
use crate::commands::ClientCommandError;

/// Control-frame failures without private commands, keys or cookie values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlMessageError {
    /// The retained server-command key is not in the reviewed baseline form.
    ServerKey,
    /// A header contains an unrepresentable ordinary-channel acknowledgement.
    Header,
    /// Movement command count, fields or profile binding was rejected.
    Movement(MovementMessageError),
    /// A command prefix could not be read from retained caller history.
    History(ClientCommandError),
    /// Checked shared message writing failed.
    Message(MessageError),
    /// Checked shared compression failed without a fitting shorter prefix.
    Compression(CompressionError),
    /// Checked shared payload obfuscation failed.
    Obfuscation(ObfuscationError),
    /// Caller output cannot hold the entire owned frame.
    OutputCapacity,
}

impl fmt::Display for ControlMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ServerKey => "invalid retained server command key",
            Self::Header => "invalid client control acknowledgement header",
            Self::Movement(_) => "invalid client movement batch",
            Self::History(_) => "client control command history unavailable",
            Self::Message(_) => "client control message encoding failed",
            Self::Compression(_) => "client control compression failed",
            Self::Obfuscation(_) => "client control payload transform failed",
            Self::OutputCapacity => "client control output capacity insufficient",
        })
    }
}

impl Error for ControlMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::History(error) => Some(error),
            Self::Movement(error) => Some(error),
            Self::Message(error) => Some(error),
            Self::Compression(error) => Some(error),
            Self::Obfuscation(error) => Some(error),
            _ => None,
        }
    }
}
