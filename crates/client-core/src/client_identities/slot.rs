// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::client_data::ClientData;
use std::fmt;

/// Opaque caller-owned retained/staging cell; unused capacity has no identity meaning.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct ClientIdentitySlot(pub(super) Option<ClientData>);
impl ClientIdentitySlot {
    pub(super) fn key(&self) -> u32 {
        // Every cell in a published used prefix is occupied; unused cells are excluded.
        self.0.as_ref().map_or(u32::MAX, ClientData::index)
    }
}
impl fmt::Debug for ClientIdentitySlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientIdentitySlot").finish_non_exhaustive()
    }
}
