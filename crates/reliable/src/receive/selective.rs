// SPDX-License-Identifier: AGPL-3.0-or-later
use super::ReceiveWindow;
use crate::{MAX_SELECTIVE_ACK_RANGES, SelectiveAckRange, SelectiveAcks};

/// One bounded page of receive acknowledgement metadata, without automatic sends.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiveAcknowledgement {
    /// First not fully consumed fragment sequence.
    pub cumulative: u32,
    /// Up to three sorted, disjoint ranges relative to the cumulative base.
    pub selective: SelectiveAcks,
    /// Resume offset for another page; zero means the window end was reached.
    /// Reset to zero when the cumulative base changes.
    pub next_offset: u16,
}

impl ReceiveWindow<'_> {
    /// Inspects the explicit host-selected span and returns up to three received runs.
    /// Includes partially read fragments because their complete original data remains
    /// retained. Offset zero begins a scan; out-of-span offsets restart at zero.
    /// Caller-held pagination prevents later runs from starving behind the first three.
    #[must_use]
    pub fn acknowledgement(&self, start_offset: u16) -> ReceiveAcknowledgement {
        let mut offset = if start_offset < self.span {
            start_offset
        } else {
            0
        };
        let mut ranges = [SelectiveAckRange::default(); MAX_SELECTIVE_ACK_RANGES];
        let mut count = 0_u8;
        while offset < self.span {
            if !self.received_at(offset) {
                offset += 1;
                continue;
            }
            let start = offset;
            while offset < self.span && self.received_at(offset) {
                offset += 1;
            }
            ranges[usize::from(count)] = SelectiveAckRange {
                offset: start,
                length: offset - start,
            };
            count += 1;
            if usize::from(count) == MAX_SELECTIVE_ACK_RANGES {
                break;
            }
        }
        ReceiveAcknowledgement {
            cumulative: self.base,
            selective: SelectiveAcks {
                ranges,
                length: count,
            },
            next_offset: if offset < self.span { offset } else { 0 },
        }
    }

    fn received_at(&self, offset: u16) -> bool {
        let index = (self.head + usize::from(offset)) % self.capacity();
        self.slots[index].sequence == Some(self.base + u32::from(offset))
    }
}
