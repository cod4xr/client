// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::MAX_COMMAND_BYTES;

use super::{ClientCommandError, LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands, history::Slot};

impl LegacyClientCommands<'_> {
    /// Queues one owned canonical command without overwriting unacknowledged work.
    /// Native byte cleaning maps 0x92 to apostrophe and percent to period; other bytes
    /// are retained. Empty commands are valid. Explicit slices are never truncated.
    ///
    /// # Errors
    /// Rejects overlong/NUL-containing input, full storage and sequence exhaustion.
    /// Every failure leaves storage and all sequence state unchanged.
    pub fn enqueue(&mut self, bytes: &[u8]) -> Result<u32, ClientCommandError> {
        if bytes.len() > MAX_COMMAND_BYTES {
            return Err(ClientCommandError::CommandTooLong);
        }
        if bytes.contains(&0) {
            return Err(ClientCommandError::EmbeddedTerminator);
        }
        if usize::try_from(self.pending_count())
            .map_err(|_| ClientCommandError::SequenceExhausted)?
            >= self.capacity
        {
            return Err(ClientCommandError::PendingCapacity);
        }
        let sequence = self
            .latest
            .checked_add(1)
            .filter(|&value| i32::try_from(value).is_ok())
            .ok_or(ClientCommandError::SequenceExhausted)?;
        let index = self.index(sequence)?;
        let start = index * LEGACY_COMMAND_SLOT_BYTES;
        let output = &mut self.pending[start..start + LEGACY_COMMAND_SLOT_BYTES];
        for (target, &byte) in output.iter_mut().zip(bytes) {
            *target = match byte {
                0x92 => b'\'',
                b'%' => b'.',
                _ => byte,
            };
        }
        output[bytes.len()] = 0;
        self.slots[index] = Slot {
            sequence,
            length: bytes.len(),
        };
        self.latest = sequence;
        Ok(sequence)
    }
}
