// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::EntityState;
use std::fmt;

/// Caller-owned final cell initialized only after complete substream validation.
/// Unused tails remain unchanged; cells carry no independent snapshot admission.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct SnapshotEntitySlot(pub(super) Option<EntityState>);
impl fmt::Debug for SnapshotEntitySlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotEntitySlot").finish_non_exhaustive()
    }
}
