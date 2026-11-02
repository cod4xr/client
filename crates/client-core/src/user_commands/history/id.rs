// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::BaselineProtocol;
use std::fmt;

/// Opaque positive local input identifier, independent of native timestamp/packet bits.
/// Its host world/profile label prevents stale references after owner reset. Hosts
/// must keep world labels unique across reconstructed/concurrent owner namespaces.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct InputCommandId {
    pub(super) epoch: u64,
    pub(super) profile: BaselineProtocol,
    pub(super) sequence: u64,
}
impl InputCommandId {
    /// Returns the full local counter, not a wire timestamp or server acknowledgement.
    #[must_use]
    pub fn sequence(self) -> u64 {
        self.sequence
    }
    /// Returns the explicit host world label, without peer identity claims.
    #[must_use]
    pub fn epoch(self) -> u64 {
        self.epoch
    }
    /// Returns the native profile selected by the original input owner.
    #[must_use]
    pub fn profile(self) -> BaselineProtocol {
        self.profile
    }
}
impl fmt::Debug for InputCommandId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputCommandId")
            .field("sequence", &self.sequence)
            .finish_non_exhaustive()
    }
}
