// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{SendError, SendSlot, SendWindow};
use crate::{MAX_RELIABLE_ACKNOWLEDGEMENT, MAX_RELIABLE_PAYLOAD_BYTES};

/// Metadata for one atomic enqueue operation; the ID interval may be empty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueuedFragments {
    /// First sequence in the half-open enqueued interval.
    pub first_sequence: u32,
    /// Sequence immediately after the enqueued interval.
    pub next_sequence: u32,
    /// Complete fragments copied from this input.
    pub fragments: usize,
    /// Complete original input bytes retained by the queue.
    pub bytes: usize,
}

impl SendWindow<'_> {
    /// Copies a complete input as consecutive up-to-1200-byte original fragments.
    /// Empty input queues nothing; exact multiples have no extra empty terminator.
    /// One failed operation changes no retained byte, tag, count or sequence.
    ///
    /// # Errors
    /// Rejects insufficient complete queue capacity and signed sequence exhaustion.
    pub fn enqueue(&mut self, input: &[u8]) -> Result<QueuedFragments, SendError> {
        let count = input.len().div_ceil(MAX_RELIABLE_PAYLOAD_BYTES);
        if count > self.capacity() - self.queued {
            return Err(SendError::Capacity);
        }
        let sequence_count = u32::try_from(count).map_err(|_| SendError::SequenceExhausted)?;
        let next = self
            .next
            .checked_add(sequence_count)
            .filter(|&value| value <= MAX_RELIABLE_ACKNOWLEDGEMENT)
            .ok_or(SendError::SequenceExhausted)?;
        let result = QueuedFragments {
            first_sequence: self.next,
            next_sequence: next,
            fragments: count,
            bytes: input.len(),
        };
        for (sequence, (offset, payload)) in
            (self.next..next).zip(input.chunks(MAX_RELIABLE_PAYLOAD_BYTES).enumerate())
        {
            let index = (self.head + self.queued + offset) % self.capacity();
            let start = index * MAX_RELIABLE_PAYLOAD_BYTES;
            self.bytes[start..start + payload.len()].copy_from_slice(payload);
            self.slots[index] = SendSlot {
                sequence: Some(sequence),
                length: payload.len(),
                transmitted: false,
                selective: false,
            };
        }
        self.next = next;
        self.queued += count;
        self.buffered += input.len();
        Ok(result)
    }
}
