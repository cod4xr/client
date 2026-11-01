// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use super::{
    SERVER_COMMAND_SLOT_BYTES, ServerCommandError, ServerCommandSlot, required_server_command_bytes,
};
use crate::{BaselineProtocol, control::ServerCommandKey, service_controls::ServerCommand};

/// Normalized command retention over caller byte/metadata storage of any chosen capacity.
/// Sequence-tagged modulo reuse prevents missing or evicted IDs from aliasing new text.
/// This stores data only; connection/reset policy and typed execution belong to the host.
pub struct ServerCommandHistory<'buffers> {
    pub(super) profile: BaselineProtocol,
    pub(super) bytes: &'buffers mut [u8],
    pub(super) slots: &'buffers mut [ServerCommandSlot],
    pub(super) newest: u32,
    pub(super) last_message: u32,
}

impl<'buffers> ServerCommandHistory<'buffers> {
    /// Borrows exact complete slots and resets metadata for a fresh connection.
    /// Dirty backing bytes are neither erased nor exposed before initialization.
    /// Choose retention according to host budgets; capacities above 128 are supported.
    ///
    /// # Errors
    /// Rejects empty metadata, multiplication overflow and mismatched byte geometry
    /// before modifying either byte storage or metadata.
    pub fn new(
        profile: BaselineProtocol,
        bytes: &'buffers mut [u8],
        slots: &'buffers mut [ServerCommandSlot],
    ) -> Result<Self, ServerCommandError> {
        if bytes.len() != required_server_command_bytes(slots.len())? {
            return Err(ServerCommandError::Storage);
        }
        slots.fill(ServerCommandSlot::default());
        Ok(Self {
            profile,
            bytes,
            slots,
            newest: 0,
            last_message: 0,
        })
    }

    /// Returns the original reviewed message profile.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns the caller-selected number of retained slots.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
    /// Returns the highest applied command sequence, initially zero.
    #[must_use]
    pub fn newest_sequence(&self) -> u32 {
        self.newest
    }
    /// Returns the last applied complete service-message sequence, initially zero.
    #[must_use]
    pub fn last_message_sequence(&self) -> u32 {
        self.last_message
    }

    /// Borrows an initialized full command by its exact original sequence.
    ///
    /// # Errors
    /// Rejects zero/future/missing/reused references without exposing backing bytes.
    pub fn command(&self, sequence: u32) -> Result<ServerCommand<'_>, ServerCommandError> {
        if sequence == 0 || sequence > self.newest {
            return Err(ServerCommandError::Missing);
        }
        let index =
            usize::try_from(sequence).map_err(|_| ServerCommandError::Sequence)? % self.slots.len();
        let slot = self.slots[index];
        if slot.sequence != sequence {
            return Err(ServerCommandError::Missing);
        }
        let start = index * SERVER_COMMAND_SLOT_BYTES;
        Ok(ServerCommand::from_stored(
            sequence,
            &self.bytes[start..start + slot.length],
        ))
    }

    /// Borrows the current original native key prefix, or the initial empty key.
    ///
    /// # Errors
    /// Rejects missing/corrupt retained metadata. Valid committed records preserve the key contract.
    pub fn latest_key(&self) -> Result<ServerCommandKey<'_>, ServerCommandError> {
        if self.newest == 0 {
            return Ok(ServerCommandKey::initial());
        }
        self.command(self.newest)?
            .key()
            .map_err(|_| ServerCommandError::CommandCapacity)
    }
}

impl fmt::Debug for ServerCommandHistory<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ServerCommandHistory")
            .field("profile", &self.profile)
            .field("capacity", &self.capacity())
            .field("newest", &self.newest)
            .field("last_message", &self.last_message)
            .finish_non_exhaustive()
    }
}
