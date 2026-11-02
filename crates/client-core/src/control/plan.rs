// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::commands::LegacyClientCommands;

use super::{
    ClientControlHeader, ControlMessageError, MAX_CONTROL_MESSAGE_BYTES, MovementBatch, assemble,
};

/// Owned reliable-command/idle/movement frame before channel headers and input admission.
/// Construction never marks commands transmitted or commits peer acknowledgements.
pub struct ControlMessagePlan {
    pub(super) bytes: [u8; MAX_CONTROL_MESSAGE_BYTES],
    pub(super) length: usize,
    pub(super) protocol: i32,
    pub(super) command_count: u32,
    pub(super) user_command_count: u8,
    pub(super) last_command: Option<u32>,
    pub(super) challenge: i32,
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
        assemble::build(header, challenge, history, None)
    }

    /// Builds a complete native movement packet with the oldest fitting reliable
    /// command prefix. Movement input is retained in full; counts are not clamped.
    /// Packet-derived keys, native profile binding and shared writer/compression/
    /// transform precede ownership of the complete private frame. This marks no
    /// input or reliable command transmitted and commits no peer acknowledgement.
    ///
    /// # Errors
    /// Rejects invalid header, movement count/fields/profile/reference binding,
    /// unavailable reliable history and shared primitive/codec exhaustion.
    pub fn with_movement(
        header: ClientControlHeader<'_>,
        challenge: i32,
        history: &LegacyClientCommands<'_>,
        movement: MovementBatch<'_, '_>,
    ) -> Result<Self, ControlMessageError> {
        assemble::build(header, challenge, history, Some(movement))
    }

    /// Returns included native movement commands, independent of reliable commands.
    #[must_use]
    pub fn user_command_count(&self) -> u8 {
        self.user_command_count
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

impl fmt::Debug for ControlMessagePlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ControlMessagePlan")
            .field("protocol", &self.protocol)
            .field("commands", &self.command_count)
            .field("user_commands", &self.user_command_count)
            .field("bytes", &self.length)
            .finish_non_exhaustive()
    }
}
