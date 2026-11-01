// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use cod4x_protocol::{
    MessageStringLimit, MessageWriter,
    compression::{self, CompressionError},
    obfuscation,
};

use crate::commands::LegacyClientCommands;

use super::{ClientControlHeader, ControlMessageError, MAX_CONTROL_MESSAGE_BYTES};

/// Owned command/idle frame, before channel headers and without gameplay movement.
/// Construction never marks commands transmitted or commits peer acknowledgements.
pub struct ControlMessagePlan {
    bytes: [u8; MAX_CONTROL_MESSAGE_BYTES],
    length: usize,
    protocol: i32,
    command_count: u32,
    last_command: Option<u32>,
    challenge: i32,
}

impl ControlMessagePlan {
    /// Builds the oldest fitting pending prefix followed by the three-bit EOF opcode.
    /// Shared compression and obfuscation apply only after the explicit plaintext header.
    /// Compression expansion retries a shorter prefix instead of clamping/truncating bytes.
    ///
    /// # Errors
    /// Rejects invalid headers and checked history/message/codec failures. Caller state
    /// remains unchanged; retained data is copied into the owned result on success.
    pub fn new(
        header: ClientControlHeader<'_>,
        challenge: i32,
        history: &LegacyClientCommands<'_>,
    ) -> Result<Self, ControlMessageError> {
        if header.message_acknowledge() > 0x7fff_ffff {
            return Err(ControlMessageError::Header);
        }
        let mut count = select_prefix(header.encoded_len(), history)?;
        let mut staging = [0; MAX_CONTROL_MESSAGE_BYTES];
        let mut plan = Self {
            bytes: [0; MAX_CONTROL_MESSAGE_BYTES],
            length: 0,
            protocol: header.requested_protocol(),
            command_count: 0,
            last_command: None,
            challenge,
        };
        loop {
            let length = write_plain(header, history, count, &mut staging)?;
            let offset = header.encoded_len();
            plan.bytes[..offset].copy_from_slice(&staging[..offset]);
            match compression::compress(&staging[offset..length], &mut plan.bytes[offset..]) {
                Ok(compressed) => {
                    plan.length = offset + compressed;
                    break;
                }
                Err(CompressionError::OutputCapacity) if count > 1 => count -= 1,
                Err(error) => return Err(ControlMessageError::Compression(error)),
            }
        }
        obfuscation::apply_client_payload(
            &mut plan.bytes[header.encoded_len()..plan.length],
            u32::from_le_bytes(challenge.to_le_bytes()),
            header.server_id(),
            header.message_acknowledge(),
            header.server_command().as_bytes(),
        )
        .map_err(ControlMessageError::Obfuscation)?;
        plan.command_count = count;
        plan.last_command = (count > 0).then(|| history.acknowledged_sequence() + count);
        Ok(plan)
    }

    /// Returns the explicit frame profile for checking against the owning connection.
    #[must_use]
    pub fn requested_protocol(&self) -> i32 {
        self.protocol
    }
    /// Returns the original compatibility cookie for connection-context checks.
    /// This is private routing/obfuscation state, not authentication; never log it.
    #[must_use]
    pub fn server_challenge(&self) -> i32 {
        self.challenge
    }
    /// Returns the command prefix included, excluding still-pending deferred commands.
    #[must_use]
    pub fn command_count(&self) -> u32 {
        self.command_count
    }
    /// Returns the highest included command. Mark this prefix transmitted only after
    /// the complete frame was admitted to outgoing I/O on its original connection.
    #[must_use]
    pub fn last_command_sequence(&self) -> Option<u32> {
        self.last_command
    }
    /// Borrows private encoded bytes for channel packetization; never log or publish them.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
    /// Returns complete frame size, excluding channel headers.
    #[must_use]
    pub fn encoded_len(&self) -> usize {
        self.length
    }

    /// Copies the complete private frame atomically into caller storage.
    ///
    /// # Errors
    /// Insufficient capacity leaves the entire output unchanged.
    pub fn encode(&self, output: &mut [u8]) -> Result<usize, ControlMessageError> {
        if output.len() < self.length {
            return Err(ControlMessageError::OutputCapacity);
        }
        output[..self.length].copy_from_slice(self.as_bytes());
        Ok(self.length)
    }
}

fn select_prefix(
    header_bytes: usize,
    history: &LegacyClientCommands<'_>,
) -> Result<u32, ControlMessageError> {
    let mut count = 0_u32;
    let mut fields = 0;
    for sequence in history.acknowledged_sequence() + 1..=history.latest_sequence() {
        let command = history
            .command(sequence)
            .map_err(ControlMessageError::History)?;
        let candidate_fields = fields + 4 + command.as_bytes().len() + 1;
        let bit_bytes = (3
            * (usize::try_from(count).map_err(|_| ControlMessageError::Header)? + 2))
            .div_ceil(8);
        if header_bytes + candidate_fields + bit_bytes > MAX_CONTROL_MESSAGE_BYTES {
            break;
        }
        fields = candidate_fields;
        count += 1;
    }
    Ok(count)
}

fn write_plain(
    header: ClientControlHeader<'_>,
    history: &LegacyClientCommands<'_>,
    count: u32,
    output: &mut [u8],
) -> Result<usize, ControlMessageError> {
    let mut writer = MessageWriter::try_new(output).map_err(ControlMessageError::Message)?;
    header
        .write(&mut writer)
        .map_err(ControlMessageError::Message)?;
    for sequence in history.acknowledged_sequence() + 1..=history.acknowledged_sequence() + count {
        let command = history
            .command(sequence)
            .map_err(ControlMessageError::History)?;
        writer
            .write_bits(2, 3)
            .map_err(ControlMessageError::Message)?;
        writer
            .write_bytes(&sequence.to_le_bytes())
            .map_err(ControlMessageError::Message)?;
        writer
            .write_legacy_string(command.as_bytes(), MessageStringLimit::Normal)
            .map_err(ControlMessageError::Message)?;
    }
    writer
        .write_bits(3, 3)
        .map_err(ControlMessageError::Message)?;
    Ok(writer.len())
}

impl fmt::Debug for ControlMessagePlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ControlMessagePlan")
            .field("protocol", &self.protocol)
            .field("commands", &self.command_count)
            .field("bytes", &self.length)
            .finish_non_exhaustive()
    }
}
