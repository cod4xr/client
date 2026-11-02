// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{SnapshotHeader, SnapshotMessage, SnapshotSource};
use crate::{
    player_state::LivePlayerState,
    snapshot_clients::{ClientStateSource, SnapshotClientSlot},
    snapshot_entities::{EntityStateSource, SnapshotEntitySlot},
};
use std::fmt;

pub(super) struct Retained {
    pub header: SnapshotHeader,
    pub message: SnapshotMessage,
    pub player: LivePlayerState,
    pub entities: usize,
    pub clients: usize,
}
/// One host-funded history slot with independently sized entity/client storage.
/// No wire packet, source plan or native ring is retained. Unused cells stay private.
pub struct SnapshotHistorySlot<'storage> {
    pub(super) entities: &'storage mut [SnapshotEntitySlot],
    pub(super) clients: &'storage mut [SnapshotClientSlot],
    pub(super) retained: Option<Retained>,
}
impl<'storage> SnapshotHistorySlot<'storage> {
    /// Reserves backing cells without clearing them or granting old data validity.
    /// Zero sparse capacity is valid for empty snapshots. Snapshot count and each
    /// slot's sparse capacities are host choices, independent of native limits.
    #[must_use]
    pub fn new(
        entities: &'storage mut [SnapshotEntitySlot],
        clients: &'storage mut [SnapshotClientSlot],
    ) -> Self {
        Self {
            entities,
            clients,
            retained: None,
        }
    }
    pub(super) fn source(&self) -> Option<SnapshotSource<'_>> {
        let retained = self.retained.as_ref()?;
        Some(SnapshotSource {
            message: retained.message,
            profile: retained.header.profile(),
            player: &retained.player,
            entities: EntityStateSource::published(&self.entities[..retained.entities]),
            clients: ClientStateSource::published(&self.clients[..retained.clients]),
        })
    }
}
impl fmt::Debug for SnapshotHistorySlot<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotHistorySlot")
            .field("entity_capacity", &self.entities.len())
            .field("client_capacity", &self.clients.len())
            .field("occupied", &self.retained.is_some())
            .finish_non_exhaustive()
    }
}
