// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{InputCommandHistory, InputCommandId, InputHistoryError, storage::Record};
use crate::user_commands::UserCommandInput;

impl InputCommandHistory<'_> {
    /// Retains one fully checked native input without overwriting unretired history.
    /// Local identifiers advance independently of wrapping or duplicate wire times.
    ///
    /// # Errors
    /// Invalid fields, full caller capacity and local sequence exhaustion preserve
    /// every slot and owner counter. No implicit retirement or timestamp policy runs.
    pub fn enqueue(
        &mut self,
        input: UserCommandInput,
    ) -> Result<InputCommandId, InputHistoryError> {
        input.validate().map_err(InputHistoryError::Command)?;
        if self.used == self.slots.len() {
            return Err(InputHistoryError::Capacity);
        }
        let sequence = self
            .latest
            .checked_add(1)
            .ok_or(InputHistoryError::SequenceExhausted)?;
        let id = self.id(sequence);
        self.slots[self.next].0 = Some(Record { id, input });
        self.next = if self.next == self.slots.len() - 1 {
            0
        } else {
            self.next + 1
        };
        self.used += 1;
        self.pending += 1;
        self.latest = sequence;
        Ok(id)
    }
    /// Copies only an exact current-namespace retained command in constant time.
    ///
    /// # Errors
    /// Stale/foreign identifiers and retired/missing tags are never substituted.
    pub fn command(&self, id: InputCommandId) -> Result<UserCommandInput, InputHistoryError> {
        self.check(id)?;
        self.record(id.sequence)
            .map(|record| record.input)
            .ok_or(InputHistoryError::Missing)
    }
    /// Resolves an exact retained local counter within this current namespace.
    /// This supports batch/replay selection without maintaining another ID queue.
    /// The supplied number is not a native timestamp or acknowledgement mapping.
    ///
    /// # Errors
    /// Retired, zero or never-retained counters are never aliased to another slot.
    pub fn id_for_local_sequence(
        &self,
        sequence: u64,
    ) -> Result<InputCommandId, InputHistoryError> {
        self.record(sequence)
            .map(|record| record.id)
            .ok_or(InputHistoryError::Missing)
    }
    pub(super) fn record(&self, sequence: u64) -> Option<&Record> {
        let offset = usize::try_from(sequence.checked_sub(self.retired)?.checked_sub(1)?).ok()?;
        if offset >= self.used {
            return None;
        }
        let oldest = if self.next >= self.used {
            self.next - self.used
        } else {
            self.slots.len() - (self.used - self.next)
        };
        let until_wrap = self.slots.len() - oldest;
        let index = if offset >= until_wrap {
            offset - until_wrap
        } else {
            oldest + offset
        };
        let record = self.slots.get(index)?.0.as_ref()?;
        (record.id == self.id(sequence)).then_some(record)
    }
    /// Releases a sent prefix under explicit host reconciliation/retention policy.
    /// This does not interpret reliable-command ACKs, snapshot command times or
    /// engine processing. Native processing/replay mapping remains host work.
    ///
    /// # Errors
    /// Foreign/stale namespaces, regression and unsent retirement preserve all data.
    /// Repeating the same retired prefix is valid and releases zero additional inputs.
    pub fn retire_through(&mut self, id: InputCommandId) -> Result<usize, InputHistoryError> {
        self.check(id)?;
        if id.sequence < self.retired || id.sequence > self.transmitted {
            return Err(InputHistoryError::Retirement);
        }
        let released = usize::try_from(id.sequence - self.retired)
            .map_err(|_| InputHistoryError::Retirement)?;
        let retained = self
            .used
            .checked_sub(released)
            .ok_or(InputHistoryError::Retirement)?;
        self.retired = id.sequence;
        self.used = retained;
        Ok(released)
    }
}
