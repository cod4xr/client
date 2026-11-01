// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::SelectiveAcks;

/// Native reliable body metadata, before temporal/window/peer validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReliableHeader {
    /// Fragment sequence starting at zero; `None` is the native ACK-only -1 sentinel.
    pub sequence: Option<u32>,
    /// Peer cumulative base, requiring validation against actually transmitted data.
    pub acknowledge: u32,
    /// Original flag bits. Only bit zero denotes force-ACK; unknown bits grant no features.
    pub flags: u8,
    /// Up to three relative acknowledgement ranges.
    pub selective: SelectiveAcks,
    /// Original signed sender-window hint. Native receivers ignore this field;
    /// parsing it never selects memory/window budgets or activates behavior.
    pub sender_window: i16,
}

impl ReliableHeader {
    /// Returns the known bit-zero signal without applying it to any runtime state.
    #[must_use]
    pub fn force_ack(&self) -> bool {
        self.flags & 1 != 0
    }
}
