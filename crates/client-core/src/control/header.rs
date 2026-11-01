// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::BaselineProtocol;
use cod4x_protocol::{MessageError, MessageWriter};

use super::ServerCommandKey;

/// Explicit source-grounded client control headers, independent of discovery hints.
#[derive(Clone, Copy)]
pub enum ClientControlHeader<'history> {
    /// Nine plaintext bytes before the compressed/transformed legacy payload.
    Legacy6 {
        /// Original legacy byte-sized server identifier.
        server_id: u8,
        /// Last validated complete ordinary server-message sequence.
        message_acknowledge: u32,
        /// Retained server command; its sequence supplies the command acknowledgement.
        server_command: ServerCommandKey<'history>,
    },
    /// Sixteen plaintext bytes before the pinned `CoD4X` 21 payload.
    Cod4x21 {
        /// Full original server identifier bits.
        server_id: u32,
        /// Last validated complete ordinary server-message sequence.
        message_acknowledge: u32,
        /// Retained server command; its sequence supplies the command acknowledgement.
        server_command: ServerCommandKey<'history>,
        /// Last validated client-config sequence for the modern header.
        config_acknowledge: u32,
    },
}

impl ClientControlHeader<'_> {
    /// Returns the explicitly chosen wire protocol, not an inferred negotiation.
    #[must_use]
    pub fn requested_protocol(&self) -> i32 {
        self.baseline_protocol().number()
    }
    /// Returns the explicit reviewed profile, without consulting remote hints.
    #[must_use]
    pub fn baseline_protocol(&self) -> BaselineProtocol {
        match self {
            Self::Legacy6 { .. } => BaselineProtocol::Legacy6,
            Self::Cod4x21 { .. } => BaselineProtocol::Cod4x21,
        }
    }
    /// Returns the complete plaintext header size.
    #[must_use]
    pub fn encoded_len(&self) -> usize {
        match self {
            Self::Legacy6 { .. } => 9,
            Self::Cod4x21 { .. } => 16,
        }
    }
    /// Returns the tracked ordinary-message acknowledgement.
    #[must_use]
    pub fn message_acknowledge(&self) -> u32 {
        match *self {
            Self::Legacy6 {
                message_acknowledge,
                ..
            }
            | Self::Cod4x21 {
                message_acknowledge,
                ..
            } => message_acknowledge,
        }
    }

    pub(super) fn server_id(&self) -> u32 {
        match *self {
            Self::Legacy6 { server_id, .. } => u32::from(server_id),
            Self::Cod4x21 { server_id, .. } => server_id,
        }
    }

    pub(super) fn server_command(&self) -> ServerCommandKey<'_> {
        match *self {
            Self::Legacy6 { server_command, .. } | Self::Cod4x21 { server_command, .. } => {
                server_command
            }
        }
    }

    pub(super) fn write(&self, writer: &mut MessageWriter<'_>) -> Result<(), MessageError> {
        match *self {
            Self::Legacy6 { server_id, .. } => writer.write_u8(server_id)?,
            Self::Cod4x21 { server_id, .. } => writer.write_bytes(&server_id.to_le_bytes())?,
        }
        writer.write_bytes(&self.message_acknowledge().to_le_bytes())?;
        writer.write_bytes(&self.server_command().sequence().to_le_bytes())?;
        if let Self::Cod4x21 {
            config_acknowledge, ..
        } = *self
        {
            writer.write_bytes(&config_acknowledge.to_le_bytes())?;
        }
        Ok(())
    }
}

impl fmt::Debug for ClientControlHeader<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientControlHeader")
            .field("protocol", &self.requested_protocol())
            .finish_non_exhaustive()
    }
}
