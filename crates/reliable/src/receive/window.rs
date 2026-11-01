// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use super::{ReceiveError, required_receive_bytes};

/// Opaque caller-owned metadata for one 1200-byte fragment slot.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReceiveSlot {
    pub(super) sequence: Option<u32>,
    pub(super) length: usize,
}

/// Bounded reordered fragments and incremental byte-stream reads over caller storage.
/// Duplicate fragments cannot replace original bytes, including a partially read head.
/// No allocation, payload logging or automatic acknowledgement transmission is owned.
pub struct ReceiveWindow<'buffers> {
    pub(super) bytes: &'buffers mut [u8],
    pub(super) slots: &'buffers mut [ReceiveSlot],
    pub(super) span: u16,
    pub(super) head: usize,
    pub(super) head_offset: usize,
    pub(super) base: u32,
    pub(super) occupied: usize,
    pub(super) buffered: usize,
}

impl<'buffers> ReceiveWindow<'buffers> {
    /// Starts a fresh zero-based window with exact byte/metadata geometry.
    /// Clears metadata only. Dirty backing bytes remain inaccessible until initialized.
    ///
    /// # Errors
    /// Rejects invalid geometry before changing either caller-provided buffer.
    pub fn new(
        bytes: &'buffers mut [u8],
        slots: &'buffers mut [ReceiveSlot],
    ) -> Result<Self, ReceiveError> {
        if bytes.len() != required_receive_bytes(slots.len())? {
            return Err(ReceiveError::Storage);
        }
        let span = u16::try_from(slots.len()).map_err(|_| ReceiveError::Storage)?;
        slots.fill(ReceiveSlot::default());
        Ok(Self {
            bytes,
            slots,
            span,
            head: 0,
            head_offset: 0,
            base: 0,
            occupied: 0,
            buffered: 0,
        })
    }

    /// Returns the host-selected fragment span, never a peer's window hint.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
    /// Returns the first not fully consumed sequence, initially zero.
    /// A partial read retains the fragment and delays cumulative acknowledgement.
    #[must_use]
    pub fn acknowledge(&self) -> u32 {
        self.base
    }
    /// Returns occupied fragment slots, including a partially read head.
    #[must_use]
    pub fn stored_fragments(&self) -> usize {
        self.occupied
    }
    /// Returns unread retained bytes, including fragments beyond a gap.
    #[must_use]
    pub fn buffered_bytes(&self) -> usize {
        self.buffered
    }
}

impl fmt::Debug for ReceiveWindow<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReceiveWindow")
            .field("capacity", &self.capacity())
            .field("acknowledge", &self.base)
            .field("fragments", &self.occupied)
            .field("buffered_bytes", &self.buffered)
            .finish_non_exhaustive()
    }
}
