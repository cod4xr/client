// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::control::{ControlMessageError, ServerCommandKey};
use cod4x_protocol::MAX_COMMAND_BYTES;
use std::fmt;

/// One complete normalized service command. No execution or deduplication is implied.
#[derive(Clone, Copy)]
pub struct ServerCommand<'commands> {
    pub(super) sequence: u32,
    pub(super) bytes: &'commands [u8],
}

impl<'commands> ServerCommand<'commands> {
    pub(crate) fn from_stored(sequence: u32, bytes: &'commands [u8]) -> Self {
        Self { sequence, bytes }
    }
    /// Returns the positive baseline command sequence.
    #[must_use]
    pub fn sequence(&self) -> u32 {
        self.sequence
    }
    /// Borrows the complete normalized big-string bytes, excluding NUL.
    /// The host must interpret these through typed game handlers; never log or execute raw text.
    #[must_use]
    pub fn as_bytes(&self) -> &'commands [u8] {
        self.bytes
    }
    /// Borrows the native retained key prefix without truncating the full command view.
    ///
    /// # Errors
    /// Propagates retained-key representation failure; validated plans preserve its contract.
    pub fn key(&self) -> Result<ServerCommandKey<'commands>, ControlMessageError> {
        ServerCommandKey::from_retained(
            self.sequence,
            &self.bytes[..self.bytes.len().min(MAX_COMMAND_BYTES)],
        )
    }
}

impl fmt::Debug for ServerCommand<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ServerCommand")
            .field("sequence", &self.sequence)
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
