// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{ReceiveSlot, ReceiveWindow};
use crate::MAX_RELIABLE_PAYLOAD_BYTES;

/// Bounded read work, without any payload or automatic acknowledgement transmission.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReadProgress {
    /// Bytes copied into the beginning of caller output.
    pub bytes: usize,
    /// Fully consumed fragments, including admitted empty fragments.
    pub fragments: usize,
    /// Ready fragments inspected, including a partially consumed head.
    pub examined: usize,
}

impl ReceiveWindow<'_> {
    /// Copies contiguous stream bytes within both output and fragment work budgets.
    /// Stops at gaps, output capacity, the explicit work quota or one window span.
    /// Zero output capacity or zero quota performs no work, even for empty fragments.
    /// A partial head remains stored until fully consumed, preserving retransmission
    /// identity and applying conservative cumulative-ACK backpressure.
    pub fn read(&mut self, output: &mut [u8], max_fragments: usize) -> ReadProgress {
        let mut progress = ReadProgress::default();
        for _ in 0..max_fragments.min(self.capacity()) {
            if progress.bytes == output.len() {
                break;
            }
            let slot = self.slots[self.head];
            if slot.sequence != Some(self.base) {
                break;
            }
            progress.examined += 1;
            let take = (slot.length - self.head_offset).min(output.len() - progress.bytes);
            let start = self.head * MAX_RELIABLE_PAYLOAD_BYTES + self.head_offset;
            output[progress.bytes..progress.bytes + take]
                .copy_from_slice(&self.bytes[start..start + take]);
            self.head_offset += take;
            self.buffered -= take;
            progress.bytes += take;
            if self.head_offset != slot.length {
                break;
            }
            self.slots[self.head] = ReceiveSlot::default();
            self.head = (self.head + 1) % self.capacity();
            self.head_offset = 0;
            self.base += 1;
            self.occupied -= 1;
            progress.fragments += 1;
        }
        progress
    }
}
