// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use cod4x_protocol::MAX_COMMAND_BYTES;

use super::ControlMessageError;

/// Borrowed retained server-command key for outbound obfuscation.
/// This validates representation only; the host must obtain it from validated
/// server-command history, not arbitrary incoming text. It grants no execution authority.
#[derive(Clone, Copy)]
pub struct ServerCommandKey<'history> {
    sequence: u32,
    bytes: &'history [u8],
}

impl<'history> ServerCommandKey<'history> {
    /// Admits an already retained baseline key, excluding its terminator.
    /// Native server-command reading replaces percent with period before retaining
    /// its first 1023 bytes. Byte 0x92 remains unchanged in this incoming direction.
    ///
    /// # Errors
    /// Rejects oversized/NUL/percent data, negative signed sequence spellings, or
    /// nonempty initial sequence-zero keys. This does not verify remote provenance.
    pub fn from_retained(
        sequence: u32,
        bytes: &'history [u8],
    ) -> Result<Self, ControlMessageError> {
        if i32::try_from(sequence).is_err()
            || bytes.len() > MAX_COMMAND_BYTES
            || bytes.contains(&0)
            || bytes.contains(&b'%')
            || (sequence == 0 && !bytes.is_empty())
        {
            return Err(ControlMessageError::ServerKey);
        }
        Ok(Self { sequence, bytes })
    }

    /// Returns the initial empty server-command key at sequence zero.
    #[must_use]
    pub fn initial() -> Self {
        Self {
            sequence: 0,
            bytes: b"",
        }
    }
    /// Returns the retained server-command sequence written into the header.
    #[must_use]
    pub fn sequence(&self) -> u32 {
        self.sequence
    }
    /// Borrows private key bytes for the shared compatibility transform; never log them.
    #[must_use]
    pub fn as_bytes(&self) -> &'history [u8] {
        self.bytes
    }
}

impl fmt::Debug for ServerCommandKey<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ServerCommandKey")
            .field("sequence", &self.sequence)
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
