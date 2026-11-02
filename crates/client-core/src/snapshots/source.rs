// SPDX-License-Identifier: AGPL-3.0-or-later
use super::SnapshotMessage;
use crate::{
    BaselineProtocol, player_state::LivePlayerState, snapshot_clients::ClientStateSource,
    snapshot_entities::EntityStateSource,
};
use std::fmt;

/// Explicit caller-owned initial baselines, before current source/order/budget checks.
#[derive(Clone, Copy)]
pub struct SnapshotBaselines<'source> {
    pub(super) epoch: u64,
    pub(super) profile: BaselineProtocol,
    pub(super) entities: EntityStateSource<'source>,
}
impl<'source> SnapshotBaselines<'source> {
    /// Labels inert initial baselines with the host's current world epoch/profile.
    /// The host must supply the actual gamestate association; labels prove no auth.
    #[must_use]
    pub fn new(
        epoch: u64,
        profile: BaselineProtocol,
        entities: EntityStateSource<'source>,
    ) -> Self {
        Self {
            epoch,
            profile,
            entities,
        }
    }
}
impl fmt::Debug for SnapshotBaselines<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotBaselines")
            .field("profile", &self.profile)
            .field("entities", &self.entities.len())
            .finish_non_exhaustive()
    }
}
/// Coherent player/entity/client references borrowed from one cooked snapshot plan.
/// Only cooked plans and retained whole snapshots construct this view; component
/// parts cannot be mixed.
#[derive(Clone, Copy)]
pub struct SnapshotSource<'snapshot> {
    pub(super) message: SnapshotMessage,
    pub(super) profile: BaselineProtocol,
    pub(super) player: &'snapshot LivePlayerState,
    pub(super) entities: EntityStateSource<'snapshot>,
    pub(super) clients: ClientStateSource<'snapshot>,
}
impl SnapshotSource<'_> {
    /// Returns the cooked reference's complete containing-message association.
    #[must_use]
    pub fn message(self) -> SnapshotMessage {
        self.message
    }
    /// Returns its explicitly selected baseline profile.
    #[must_use]
    pub fn profile(self) -> BaselineProtocol {
        self.profile
    }
}
impl fmt::Debug for SnapshotSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotSource")
            .field("profile", &self.profile)
            .field("entities", &self.entities.len())
            .field("clients", &self.clients.len())
            .finish_non_exhaustive()
    }
}
