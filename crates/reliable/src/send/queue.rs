// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{SendError, SendSlot, SendWindow};
use crate::framing::ReliableFrame;
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
        self.enqueue_parts(input, b"")
    }

    /// Atomically copies a complete frame prefix/body directly into outgoing slots.
    /// No full-message temporary allocation or partially queued prefix is required.
    ///
    /// # Errors
    /// Rejects prefix representation, complete queue capacity and sequence exhaustion
    /// before changing any retained bytes, tags or counters.
    pub fn enqueue_frame(
        &mut self,
        frame: ReliableFrame<'_>,
    ) -> Result<QueuedFragments, SendError> {
        let prefix = frame.length_prefix().map_err(SendError::Frame)?;
        self.enqueue_parts(&prefix, frame.body())
    }

    fn enqueue_parts(&mut self, first: &[u8], second: &[u8]) -> Result<QueuedFragments, SendError> {
        let length = first
            .len()
            .checked_add(second.len())
            .ok_or(SendError::Capacity)?;
        let count = length.div_ceil(MAX_RELIABLE_PAYLOAD_BYTES);
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
            bytes: length,
        };
        for (sequence, offset) in
            (self.next..next).zip((0..length).step_by(MAX_RELIABLE_PAYLOAD_BYTES))
        {
            let index =
                (self.head + self.queued + (offset / MAX_RELIABLE_PAYLOAD_BYTES)) % self.capacity();
            let start = index * MAX_RELIABLE_PAYLOAD_BYTES;
            let take = (length - offset).min(MAX_RELIABLE_PAYLOAD_BYTES);
            let from_first = first.len().saturating_sub(offset).min(take);
            let output = &mut self.bytes[start..start + take];
            if from_first != 0 {
                output[..from_first].copy_from_slice(&first[offset..offset + from_first]);
            }
            if from_first != take {
                let from = offset + from_first - first.len();
                output[from_first..].copy_from_slice(&second[from..from + take - from_first]);
            }
            self.slots[index] = SendSlot {
                sequence: Some(sequence),
                length: take,
                transmitted: false,
                selective: false,
            };
        }
        self.next = next;
        self.queued += count;
        self.buffered += length;
        Ok(result)
    }
}
