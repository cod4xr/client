// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use super::{
    ClientCommand, ClientCommandError, LEGACY_COMMAND_SLOT_BYTES, MAX_LEGACY_COMMAND_STORAGE_BYTES,
    MAX_LEGACY_PENDING_COMMANDS,
};

#[derive(Clone, Copy, Default)]
pub(super) struct Slot {
    pub(super) sequence: u32,
    pub(super) length: usize,
}

/// Socket-free baseline client-command history over explicitly supplied storage.
/// One separate key slot survives pending-ring reuse. No private bytes enter Debug.
pub struct LegacyClientCommands<'storage> {
    pub(super) key: &'storage mut [u8],
    pub(super) pending: &'storage mut [u8],
    pub(super) slots: [Slot; MAX_LEGACY_PENDING_COMMANDS],
    pub(super) capacity: usize,
    pub(super) latest: u32,
    pub(super) transmitted: u32,
    pub(super) acknowledged: u32,
    pub(super) key_length: usize,
}

impl<'storage> LegacyClientCommands<'storage> {
    /// Borrows one key slot plus 1–128 whole command slots. Dirty backing bytes are
    /// never exposed without initialization; setup does not erase caller storage.
    ///
    /// # Errors
    /// Rejects incomplete slots and capacity outside the named baseline profile.
    pub fn new(storage: &'storage mut [u8]) -> Result<Self, ClientCommandError> {
        if storage.len() < 2 * LEGACY_COMMAND_SLOT_BYTES
            || storage.len() > MAX_LEGACY_COMMAND_STORAGE_BYTES
            || !storage.len().is_multiple_of(LEGACY_COMMAND_SLOT_BYTES)
        {
            return Err(ClientCommandError::Storage);
        }
        let capacity = storage.len() / LEGACY_COMMAND_SLOT_BYTES - 1;
        let (key, pending) = storage.split_at_mut(LEGACY_COMMAND_SLOT_BYTES);
        Ok(Self {
            key,
            pending,
            slots: [Slot::default(); MAX_LEGACY_PENDING_COMMANDS],
            capacity,
            latest: 0,
            transmitted: 0,
            acknowledged: 0,
            key_length: 0,
        })
    }

    /// Returns chosen pending capacity; the separately retained key does not consume it.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    /// Returns the highest successfully queued sequence, initially zero.
    #[must_use]
    pub fn latest_sequence(&self) -> u32 {
        self.latest
    }
    /// Returns the highest prefix admitted to outgoing I/O by the host.
    #[must_use]
    pub fn transmitted_sequence(&self) -> u32 {
        self.transmitted
    }
    /// Returns the committed peer acknowledgement, initially the empty key at zero.
    #[must_use]
    pub fn acknowledged_sequence(&self) -> u32 {
        self.acknowledged
    }
    /// Returns unacknowledged queued commands, including unsent commands.
    #[must_use]
    pub fn pending_count(&self) -> u32 {
        self.latest - self.acknowledged
    }

    /// Borrows the current acknowledged key or a retained pending command.
    /// This permits outgoing encoding; use `check_acknowledgement` for peer numbers.
    ///
    /// # Errors
    /// Rejects old, future or reused slot references without returning backing bytes.
    pub fn command(&self, sequence: u32) -> Result<ClientCommand<'_>, ClientCommandError> {
        let bytes = if sequence == self.acknowledged {
            &self.key[..self.key_length]
        } else {
            let (index, length) = self.position(sequence)?;
            let start = index * LEGACY_COMMAND_SLOT_BYTES;
            &self.pending[start..start + length]
        };
        Ok(ClientCommand { sequence, bytes })
    }

    pub(super) fn index(&self, sequence: u32) -> Result<usize, ClientCommandError> {
        Ok(
            usize::try_from(sequence).map_err(|_| ClientCommandError::SequenceExhausted)?
                % self.capacity,
        )
    }

    pub(super) fn position(&self, sequence: u32) -> Result<(usize, usize), ClientCommandError> {
        if sequence <= self.acknowledged || sequence > self.latest {
            return Err(ClientCommandError::UnretainedSequence);
        }
        let index = self.index(sequence)?;
        let slot = self.slots[index];
        if slot.sequence != sequence {
            return Err(ClientCommandError::UnretainedSequence);
        }
        Ok((index, slot.length))
    }
}

impl fmt::Debug for LegacyClientCommands<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyClientCommands")
            .field("capacity", &self.capacity)
            .field("latest", &self.latest)
            .field("transmitted", &self.transmitted)
            .field("acknowledged", &self.acknowledged)
            .finish_non_exhaustive()
    }
}
