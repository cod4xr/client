// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use super::{SendError, SendSlot, SendWindow};
use crate::{MAX_SELECTIVE_ACK_RANGES, ReliablePacket};

#[derive(Clone, Copy, Default)]
struct AckSpan {
    first: u32,
    end: u32,
    index: usize,
    count: usize,
}

/// Whole preflighted peer ACK effect, applied only by explicit commit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcknowledgementUpdate {
    /// New consumed cumulative base.
    pub acknowledge: u32,
    /// Original fragments released by the cumulative prefix.
    pub released_fragments: usize,
    /// Original payload bytes released by that prefix.
    pub released_bytes: usize,
    /// Unique newly selectively acknowledged fragments, retaining their payload storage.
    pub newly_selective: usize,
}

/// Exclusive whole-ACK preparation. Drop cancels all cumulative/selective changes.
/// Prepare the incoming receive update separately before committing both infallibly.
#[must_use = "commit only after whole-packet/receive validation; dropping cancels the ACK update"]
pub struct PreparedAcknowledgement<'window, 'buffers> {
    owner: &'window mut SendWindow<'buffers>,
    ranges: [AckSpan; MAX_SELECTIVE_ACK_RANGES],
    range_count: usize,
    summary: AcknowledgementUpdate,
}

impl<'buffers> SendWindow<'buffers> {
    /// Preflights all cumulative/selective ACKs from a completely parsed packet.
    /// Every claimed fragment must have been actually transmitted, not merely queued.
    /// Flag bits, incoming sender hints and incoming data are not applied here.
    ///
    /// # Errors
    /// Rejects regressing/future ACKs, ranges beyond current retained/transmitted data
    /// and unsent holes. Any late failure leaves all bytes/tags/counters unchanged.
    pub fn prepare_acknowledgement(
        &mut self,
        packet: ReliablePacket<'_>,
    ) -> Result<PreparedAcknowledgement<'_, 'buffers>, SendError> {
        let header = packet.header();
        let acknowledge = header.acknowledge;
        if acknowledge < self.base {
            return Err(SendError::AckRegression);
        }
        if acknowledge > self.next || acknowledge - self.base > u32::from(self.span) {
            return Err(SendError::AckFuture);
        }
        let released =
            usize::try_from(acknowledge - self.base).map_err(|_| SendError::AckFuture)?;
        let mut released_bytes = 0;
        for offset in 0..released {
            let slot = self.slots[(self.head + offset) % self.capacity()];
            if !slot.transmitted {
                return Err(SendError::AckUnsent);
            }
            released_bytes += slot.length;
        }
        let mut ranges = [AckSpan::default(); MAX_SELECTIVE_ACK_RANGES];
        let mut newly_selective = 0;
        for (ordinal, range) in header.selective.ranges().iter().enumerate() {
            let first = acknowledge
                .checked_add(u32::from(range.offset))
                .ok_or(SendError::AckRange)?;
            let end = first
                .checked_add(u32::from(range.length))
                .ok_or(SendError::AckRange)?;
            if end > self.next || end - acknowledge > u32::from(self.span) {
                return Err(SendError::AckRange);
            }
            let index = self.index(first).map_err(|_| SendError::AckRange)?;
            for relative in 0..range.length {
                let sequence = first + u32::from(relative);
                let slot = self.slots[(index + usize::from(relative)) % self.capacity()];
                if !slot.transmitted {
                    return Err(SendError::AckUnsent);
                }
                let repeated = ranges[..ordinal]
                    .iter()
                    .any(|earlier| earlier.first <= sequence && sequence < earlier.end);
                if !slot.selective && !repeated {
                    newly_selective += 1;
                }
            }
            ranges[ordinal] = AckSpan {
                first,
                end,
                index,
                count: usize::from(range.length),
            };
        }
        Ok(PreparedAcknowledgement {
            owner: self,
            ranges,
            range_count: header.selective.len(),
            summary: AcknowledgementUpdate {
                acknowledge,
                released_fragments: released,
                released_bytes,
                newly_selective,
            },
        })
    }
}

impl PreparedAcknowledgement<'_, '_> {
    /// Returns prospective bounded counters only, without changing queue state.
    #[must_use]
    pub fn summary(&self) -> AcknowledgementUpdate {
        self.summary
    }
    /// Applies every preflighted selective mark and cumulative release infallibly.
    /// Selective ACKs suppress retransmission but retain bytes until cumulative consumption.
    #[must_use]
    pub fn commit(self) -> AcknowledgementUpdate {
        let capacity = self.owner.capacity();
        for range in &self.ranges[..self.range_count] {
            for offset in 0..range.count {
                self.owner.slots[(range.index + offset) % capacity].selective = true;
            }
        }
        for offset in 0..self.summary.released_fragments {
            self.owner.slots[(self.owner.head + offset) % capacity] = SendSlot::default();
        }
        self.owner.head = (self.owner.head + self.summary.released_fragments) % capacity;
        self.owner.base = self.summary.acknowledge;
        self.owner.queued -= self.summary.released_fragments;
        self.owner.buffered -= self.summary.released_bytes;
        self.summary
    }
}

impl fmt::Debug for PreparedAcknowledgement<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedAcknowledgement")
            .field("summary", &self.summary)
            .finish_non_exhaustive()
    }
}
