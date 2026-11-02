// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    InputCommandHistory, InputCommandId, InputHistoryError, InputTransmission, PreparedInputBatch,
};
use crate::user_commands::MAX_PACKET_USER_COMMANDS;

impl<'storage> InputCommandHistory<'storage> {
    /// Copies an exact contiguous native packet range while leasing the original
    /// input owner. Previously sent backups are allowed; new input cannot skip an
    /// unsent prefix. Dropping cancels transmission metadata without changing slots.
    ///
    /// # Errors
    /// Rejects namespace/range/count/retention failures and skipped pending inputs.
    /// Every failure preserves all owner counters and physical storage.
    pub fn prepare_range<'owner>(
        &'owner mut self,
        first: InputCommandId,
        last: InputCommandId,
    ) -> Result<PreparedInputBatch<'owner, 'storage>, InputHistoryError> {
        self.check(first)?;
        self.check(last)?;
        if first.sequence == 0 || first.sequence > last.sequence {
            return Err(InputHistoryError::BatchRange);
        }
        if first.sequence <= self.retired || last.sequence > self.latest {
            return Err(InputHistoryError::Missing);
        }
        let count = last
            .sequence
            .checked_sub(first.sequence)
            .and_then(|distance| distance.checked_add(1))
            .ok_or(InputHistoryError::BatchRange)?;
        if count
            > u64::try_from(MAX_PACKET_USER_COMMANDS).map_err(|_| InputHistoryError::BatchCount)?
        {
            return Err(InputHistoryError::BatchCount);
        }
        if last.sequence > self.transmitted
            && self
                .transmitted
                .checked_add(1)
                .is_none_or(|next| first.sequence > next)
        {
            return Err(InputHistoryError::TransmissionGap);
        }
        let count = usize::try_from(count).map_err(|_| InputHistoryError::BatchCount)?;
        let initial = self
            .record(first.sequence)
            .ok_or(InputHistoryError::Missing)?
            .input;
        let mut commands = [initial; MAX_PACKET_USER_COMMANDS];
        for (offset, destination) in commands[..count].iter_mut().enumerate() {
            let sequence = first
                .sequence
                .checked_add(u64::try_from(offset).map_err(|_| InputHistoryError::BatchRange)?)
                .ok_or(InputHistoryError::BatchRange)?;
            *destination = self
                .record(sequence)
                .ok_or(InputHistoryError::Missing)?
                .input;
        }
        let through = self.transmitted.max(last.sequence);
        let newly_transmitted = usize::try_from(through - self.transmitted)
            .map_err(|_| InputHistoryError::TransmissionGap)?;
        let pending_after = self
            .pending
            .checked_sub(newly_transmitted)
            .ok_or(InputHistoryError::TransmissionGap)?;
        let summary = InputTransmission {
            first,
            last,
            through: self.id(through),
            command_count: count,
            newly_transmitted,
        };
        Ok(PreparedInputBatch {
            owner: self,
            commands,
            summary,
            pending_after,
        })
    }
}
