// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use super::{DecodedServerPayload, SERVER_ACKNOWLEDGEMENT_BYTES, ServerPayloadError};
use crate::{BaselineProtocol, commands::LegacyClientCommands};
use cod4x_protocol::{MAX_MESSAGE_BYTES, MessageReader, compression, obfuscation};

/// Socket-free ordinary server-body decoder with original private connection context.
/// It owns no allocator/buffers and never commits acknowledgement or executes commands.
pub struct ServerPayloadDecoder {
    profile: BaselineProtocol,
    challenge: i32,
}

impl ServerPayloadDecoder {
    /// Selects the original explicit message profile and compatibility cookie.
    /// Neither value is inferred from an incoming packet or discovery advertisement.
    #[must_use]
    pub fn new(profile: BaselineProtocol, challenge: i32) -> Self {
        Self { profile, challenge }
    }

    /// Checks a retained transmitted key, transforms scratch and decompresses output.
    /// Supply one complete body from the selected peer's established ordinary channel.
    /// Returned data still requires full service-opcode/EOF validation before history
    /// commit. Output and history remain unchanged on failure; private scratch contents
    /// are unspecified after transform begins and must never be logged or consumed then.
    ///
    /// # Errors
    /// Rejects sequence/header/input/scratch limits, invalid peer acknowledgement and
    /// shared codec errors. Decompression output is bounded by the baseline 128 KiB profile.
    ///
    /// The command key cannot be released while the decoded view is still inspected:
    /// ```compile_fail
    /// use cod4xr_client_core::{commands::LegacyClientCommands, server_payload::ServerPayloadDecoder};
    /// fn release_early(history: &mut LegacyClientCommands<'_>, decoder: &ServerPayloadDecoder,
    ///     work: &mut [u8], output: &mut [u8]) {
    ///     if let Ok(message) = decoder.decode(7, &[0, 0, 0, 0, 0x9e, 0x7f], history, work, output) {
    ///         let number = message.client_command_acknowledge();
    ///         let body = message.body();
    ///         let _ = history.commit_acknowledgement(number);
    ///         let _ = body.len();
    ///     }
    /// }
    /// ```
    pub fn decode<'decoded, 'history>(
        &self,
        sequence: u32,
        packet: &[u8],
        history: &'history LegacyClientCommands<'_>,
        work: &mut [u8],
        output: &'decoded mut [u8],
    ) -> Result<DecodedServerPayload<'decoded, 'history>, ServerPayloadError> {
        if sequence == 0 || sequence > 0x7fff_ffff {
            return Err(ServerPayloadError::Sequence);
        }
        if packet.len() > MAX_MESSAGE_BYTES {
            return Err(ServerPayloadError::InputCapacity);
        }
        let mut reader = MessageReader::try_new(packet).map_err(|_| ServerPayloadError::Header)?;
        let number = reader.read_i32().map_err(|_| ServerPayloadError::Header)?;
        if packet.len() == SERVER_ACKNOWLEDGEMENT_BYTES {
            return Err(ServerPayloadError::Empty);
        }
        let acknowledgement = history
            .check_acknowledgement(u32::from_le_bytes(number.to_le_bytes()))
            .map_err(ServerPayloadError::Acknowledgement)?;
        let encrypted = &packet[SERVER_ACKNOWLEDGEMENT_BYTES..];
        if work.len() < encrypted.len() {
            return Err(ServerPayloadError::WorkCapacity);
        }
        let transformed = &mut work[..encrypted.len()];
        transformed.copy_from_slice(encrypted);
        obfuscation::apply_server_payload(
            transformed,
            u32::from_le_bytes(self.challenge.to_le_bytes()),
            sequence,
            acknowledgement.command().as_bytes(),
        )
        .map_err(ServerPayloadError::Obfuscation)?;
        let length = compression::decompress(transformed, output)
            .map_err(ServerPayloadError::Compression)?;
        Ok(DecodedServerPayload {
            profile: self.profile,
            sequence,
            acknowledgement,
            body: &output[..length],
        })
    }
}

impl fmt::Debug for ServerPayloadDecoder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ServerPayloadDecoder")
            .field("profile", &self.profile)
            .finish_non_exhaustive()
    }
}
