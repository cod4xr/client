// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{InputCommandId, InputCommandSlot, InputHistoryError};
use crate::BaselineProtocol;
use std::fmt;

/// Current-world input/replay history with caller-selected capacity and exact tags.
/// Local counters, transmitted prefix and explicitly retired prefix are distinct.
/// Native packet count and timestamp widths do not set this owner's capacity.
pub struct InputCommandHistory<'storage> {
    pub(super) slots: &'storage mut [InputCommandSlot],
    pub(super) profile: BaselineProtocol,
    pub(super) epoch: u64,
    pub(super) next: usize,
    pub(super) used: usize,
    pub(super) pending: usize,
    pub(super) latest: u64,
    pub(super) transmitted: u64,
    pub(super) retired: u64,
}
impl<'storage> InputCommandHistory<'storage> {
    /// Starts a fresh host world/profile without exposing or clearing old slot data.
    /// Construction/reset are constant-time logical invalidations. World-label
    /// uniqueness across newly constructed owners is the host's responsibility.
    ///
    /// # Errors
    /// Empty storage rejects without modifying any cell.
    pub fn new(
        epoch: u64,
        profile: BaselineProtocol,
        slots: &'storage mut [InputCommandSlot],
    ) -> Result<Self, InputHistoryError> {
        if slots.is_empty() {
            return Err(InputHistoryError::Storage);
        }
        Ok(Self {
            slots,
            profile,
            epoch,
            next: 0,
            used: 0,
            pending: 0,
            latest: 0,
            transmitted: 0,
            retired: 0,
        })
    }
    /// Logically discards the old world and its pending inputs for an explicit reset.
    /// Existing caller-held identifiers cannot select inputs from the new namespace.
    /// Retained physical cells stay private until overwritten by future enqueue.
    ///
    /// # Errors
    /// Equal/older epochs preserve the complete owner and all cells.
    pub fn reset(
        &mut self,
        epoch: u64,
        profile: BaselineProtocol,
    ) -> Result<(), InputHistoryError> {
        if epoch <= self.epoch {
            return Err(InputHistoryError::EpochOrder);
        }
        self.epoch = epoch;
        self.profile = profile;
        self.next = 0;
        self.used = 0;
        self.pending = 0;
        self.latest = 0;
        self.transmitted = 0;
        self.retired = 0;
        Ok(())
    }
    /// Returns the explicit host world label, not peer identity.
    #[must_use]
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    /// Returns this owner's selected native profile.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns caller-funded retained inputs, independently of native packet count.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
    /// Returns retained sent and pending inputs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.used
    }
    /// Reports whether no input is retained, even if older counters remain.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.used == 0
    }
    /// Returns queued inputs not yet admitted to complete transmission.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending
    }
    /// Returns the latest created local identifier, including explicitly retired data.
    #[must_use]
    pub fn latest_id(&self) -> Option<InputCommandId> {
        (self.latest != 0).then(|| self.id(self.latest))
    }
    /// Returns the admitted transmitted prefix, not proof of native processing.
    #[must_use]
    pub fn transmitted_id(&self) -> Option<InputCommandId> {
        (self.transmitted != 0).then(|| self.id(self.transmitted))
    }
    /// Returns the host's explicitly retired prefix, not a reliable-command ACK.
    #[must_use]
    pub fn retired_id(&self) -> Option<InputCommandId> {
        (self.retired != 0).then(|| self.id(self.retired))
    }
    pub(super) fn id(&self, sequence: u64) -> InputCommandId {
        InputCommandId {
            epoch: self.epoch,
            profile: self.profile,
            sequence,
        }
    }
    pub(super) fn check(&self, id: InputCommandId) -> Result<(), InputHistoryError> {
        if id.epoch != self.epoch || id.profile != self.profile {
            return Err(InputHistoryError::Binding);
        }
        Ok(())
    }
}
impl fmt::Debug for InputCommandHistory<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputCommandHistory")
            .field("profile", &self.profile)
            .field("capacity", &self.slots.len())
            .field("retained", &self.used)
            .field("pending", &self.pending)
            .finish_non_exhaustive()
    }
}
