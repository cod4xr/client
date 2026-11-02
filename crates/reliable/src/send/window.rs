// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, ops::Range};

use super::{MAX_SEND_WINDOW_FRAGMENTS, SendError, required_send_bytes};

/// Opaque host-owned metadata for one retained 1200-byte outgoing fragment slot.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SendSlot {
    pub(super) sequence: Option<u32>,
    pub(super) length: usize,
    pub(super) transmitted: bool,
    pub(super) selective: bool,
}

/// Dense sequence-tagged outgoing queue over fixed caller byte/metadata buffers.
/// Queued storage and the in-flight span are separate host resource decisions.
pub struct SendWindow<'buffers> {
    pub(super) bytes: &'buffers mut [u8],
    pub(super) slots: &'buffers mut [SendSlot],
    pub(super) span: u16,
    pub(super) hint: i16,
    pub(super) head: usize,
    pub(super) base: u32,
    pub(super) next: u32,
    pub(super) queued: usize,
    pub(super) buffered: usize,
}

impl<'buffers> SendWindow<'buffers> {
    /// Starts a zero-based queue, clearing metadata only after complete geometry checks.
    /// Dirty payload storage remains inaccessible until initialized by enqueue.
    ///
    /// # Errors
    /// Rejects empty/mismatched storage, multiplication overflow and a zero,
    /// unrepresentable or unfunded in-flight span without changing caller storage.
    pub fn new(
        bytes: &'buffers mut [u8],
        slots: &'buffers mut [SendSlot],
        span: u16,
    ) -> Result<Self, SendError> {
        let hint = Self::validate_geometry(bytes.len(), slots.len(), span)?;
        slots.fill(SendSlot::default());
        Ok(Self {
            bytes,
            slots,
            span,
            hint,
            head: 0,
            base: 0,
            next: 0,
            queued: 0,
            buffered: 0,
        })
    }

    pub(crate) fn validate_geometry(
        bytes: usize,
        slots: usize,
        span: u16,
    ) -> Result<i16, SendError> {
        if bytes != required_send_bytes(slots)?
            || span == 0
            || span > MAX_SEND_WINDOW_FRAGMENTS
            || usize::from(span) > slots
        {
            return Err(SendError::Storage);
        }
        i16::try_from(span).map_err(|_| SendError::Storage)
    }

    /// Returns total host-funded queued fragment slots.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
    /// Returns the explicit current in-flight span, independent of peer hints.
    #[must_use]
    pub fn in_flight_limit(&self) -> u16 {
        self.span
    }
    /// Returns the released cumulative base, initially zero.
    #[must_use]
    pub fn acknowledge(&self) -> u32 {
        self.base
    }
    /// Returns the next queue sequence, initially zero.
    #[must_use]
    pub fn next_sequence(&self) -> u32 {
        self.next
    }
    /// Returns queued fragments, including transmitted/selectively acknowledged ones.
    #[must_use]
    pub fn queued_fragments(&self) -> usize {
        self.queued
    }
    /// Returns retained original payload bytes, freed only by a validated cumulative ACK.
    #[must_use]
    pub fn buffered_bytes(&self) -> usize {
        self.buffered
    }
    /// Returns at most the explicit in-flight span of candidate IDs. Selectively
    /// acknowledged IDs remain in this range but transmission preparation rejects them.
    /// This returns metadata only and grants no automatic send/pacing permission.
    #[must_use]
    pub fn active_sequences(&self) -> Range<u32> {
        self.base..self.base + (self.next - self.base).min(u32::from(self.span))
    }

    /// Selects an eligible active ID in round-robin order without copying payloads.
    /// Missing/out-of-span cursors restart at the current cumulative base.
    /// Selectively acknowledged data is skipped, including a wholly suppressed span.
    ///
    /// # Errors
    /// Propagates invalid retained-ID/host-index conversion instead of aliasing another slot.
    pub fn next_transmission_sequence(&self, cursor: u32) -> Result<Option<u32>, SendError> {
        let active = self.active_sequences();
        let start = if active.contains(&cursor) {
            cursor
        } else {
            active.start
        };
        for sequence in (start..active.end).chain(active.start..start) {
            let index = self.index(sequence)?;
            if !self.slots[index].selective {
                return Ok(Some(sequence));
            }
        }
        Ok(None)
    }

    pub(super) fn index(&self, sequence: u32) -> Result<usize, SendError> {
        let distance = sequence.checked_sub(self.base).ok_or(SendError::Missing)?;
        let distance = usize::try_from(distance).map_err(|_| SendError::Missing)?;
        if distance >= self.queued {
            return Err(SendError::Missing);
        }
        let index = (self.head + distance) % self.capacity();
        if self.slots[index].sequence != Some(sequence) {
            return Err(SendError::Missing);
        }
        Ok(index)
    }
}

impl fmt::Debug for SendWindow<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SendWindow")
            .field("capacity", &self.capacity())
            .field("in_flight_limit", &self.span)
            .field("acknowledge", &self.base)
            .field("next_sequence", &self.next)
            .field("queued_fragments", &self.queued)
            .field("buffered_bytes", &self.buffered)
            .finish_non_exhaustive()
    }
}
