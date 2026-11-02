// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{SnapshotError, SnapshotHeader, SnapshotMessage, SnapshotOutput, SnapshotPlan};
use crate::{
    message_body::BodyReader,
    player_state::LivePlayerState,
    snapshot_clients::{
        ClientStreamStats, SnapshotClientPlan, sink as client_sink, walk as client_walk,
    },
    snapshot_entities::{
        EntityStreamStats, SnapshotEntityPlan, sink as entity_sink, walk as entity_walk,
    },
};

pub(super) struct Transaction<'reader, 'body, 'source, 'storage> {
    pub reader: &'reader mut BodyReader<'body>,
    pub candidate: BodyReader<'body>,
    pub stream_start: BodyReader<'body>,
    pub header: SnapshotHeader,
    pub message: SnapshotMessage,
    pub output: SnapshotOutput<'storage>,
    pub entity_input: entity_walk::Input<'source>,
    pub client_input: client_walk::Input<'source>,
    pub entity_stats: EntityStreamStats,
    pub client_stats: ClientStreamStats,
    pub entity_bytes: usize,
    pub client_bytes: usize,
    pub consumed_bytes: usize,
}
impl<'storage> Transaction<'_, '_, '_, 'storage> {
    pub fn finish(self, player: LivePlayerState) -> Result<SnapshotPlan<'storage>, SnapshotError> {
        let SnapshotOutput { entities, clients } = self.output;
        let mut replay = self.stream_start;
        // Both streams were completely validated before any output write. Only
        // closed replay runs here; all bytes/sources/cursors/context/budgets and
        // capacities are identical, and prediction was resolved before entering.
        entity_walk::walk(
            &mut replay,
            self.entity_input,
            &mut entity_sink::Writer {
                slots: entities,
                written: 0,
            },
        )
        .map_err(SnapshotError::Entities)?;
        client_walk::walk(
            &mut replay,
            self.client_input,
            &mut client_sink::Writer {
                slots: clients,
                written: 0,
            },
        )
        .map_err(SnapshotError::Clients)?;
        let entities = SnapshotEntityPlan::published(
            self.header.profile(),
            &entities[..self.entity_stats.output_entities],
            self.entity_stats,
            self.entity_bytes,
        );
        let clients = SnapshotClientPlan::published(
            self.header.profile(),
            &clients[..self.client_stats.output_clients],
            self.client_stats,
            self.client_bytes,
        );
        *self.reader = self.candidate;
        Ok(SnapshotPlan {
            header: self.header,
            message: self.message,
            player,
            entities,
            clients,
            consumed_bytes: self.consumed_bytes,
        })
    }
}
