// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::ClientState;
use std::fmt;

/// Caller-owned final cell published only after complete substream validation.
/// Unused output tails remain unchanged. This grants no full snapshot admission.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct SnapshotClientSlot(pub(super) Option<ClientState>);
impl fmt::Debug for SnapshotClientSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotClientSlot").finish_non_exhaustive()
    }
}
