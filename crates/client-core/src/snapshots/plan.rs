// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    PreparedSnapshot, SnapshotError, SnapshotHeader, SnapshotInput, SnapshotMessage,
    SnapshotOutput, SnapshotSource, prepare,
};
use crate::{
    message_body::BodyReader, player_state::LivePlayerState, snapshot_clients::SnapshotClientPlan,
    snapshot_entities::SnapshotEntityPlan,
};
use std::fmt;

/// Complete native snapshot wire projection, before host history/message admission.
/// All parts belong to one explicit world/profile/message association.
pub struct SnapshotPlan<'storage> {
    pub(super) header: SnapshotHeader,
    pub(super) message: SnapshotMessage,
    pub(super) player: LivePlayerState,
    pub(super) entities: SnapshotEntityPlan<'storage>,
    pub(super) clients: SnapshotClientPlan<'storage>,
    pub(super) consumed_bytes: usize,
}
impl<'storage> SnapshotPlan<'storage> {
    /// Validates header, complete player state and both sparse streams before any
    /// final output write. An exclusive reader/output lease holds immutable replay
    /// inputs until explicit prediction resolution and whole-snapshot commitment.
    /// Starts immediately after service opcode six; following service data remains.
    /// No peer-selected allocation, host callback, global ring or I/O runs here.
    ///
    /// # Errors
    /// Rejects binding/source/budget/capacity failures, malformed/truncated fields
    /// and missing framing without changing either output buffer or reader cursor.
    /// Prediction failures and dropping the returned lease also preserve all three.
    pub fn prepare<'reader, 'body, 'source>(
        reader: &'reader mut BodyReader<'body>,
        input: SnapshotInput<'source>,
        output: SnapshotOutput<'storage>,
    ) -> Result<PreparedSnapshot<'reader, 'body, 'source, 'storage>, SnapshotError> {
        prepare::prepare(reader, input, output)
    }
    /// Returns the complete inert header, excluding any service tag.
    #[must_use]
    pub fn header(&self) -> SnapshotHeader {
        self.header
    }
    /// Returns the caller-associated world/message/server-command metadata.
    #[must_use]
    pub fn message(&self) -> SnapshotMessage {
        self.message
    }
    /// Borrows the complete prediction-resolved player-state wire projection.
    #[must_use]
    pub fn player(&self) -> &LivePlayerState {
        &self.player
    }
    /// Borrows the complete cooked entity substream.
    #[must_use]
    pub fn entities(&self) -> &SnapshotEntityPlan<'storage> {
        &self.entities
    }
    /// Borrows the complete cooked client-state substream.
    #[must_use]
    pub fn clients(&self) -> &SnapshotClientPlan<'storage> {
        &self.clients
    }
    /// Returns newly consumed physical bytes; cached bits can also advance.
    #[must_use]
    pub fn consumed_bytes(&self) -> usize {
        self.consumed_bytes
    }
    /// Borrows all parts as one coherent future reference without retaining source
    /// wire/prior borrows. The view keeps this plan and its backing cells immutable.
    ///
    /// ```compile_fail,E0499
    /// use cod4xr_client_core::{message_body::BodyReader,
    ///     snapshots::{SnapshotInput, SnapshotOutput, SnapshotPlan, SnapshotError}};
    /// fn reference_lease(reader: &mut BodyReader<'_>, input: SnapshotInput<'_>, output: SnapshotOutput<'_>)
    ///     -> Result<(), SnapshotError> {
    ///     let SnapshotOutput { entities, clients } = output;
    ///     let old = SnapshotPlan::prepare(reader, input,
    ///         SnapshotOutput { entities: &mut *entities, clients: &mut *clients })?.commit_transmitted()?;
    ///     let source = old.as_source();
    ///     entities.fill(Default::default());
    ///     let _ = source.message();
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn as_source(&self) -> SnapshotSource<'_> {
        SnapshotSource {
            message: self.message,
            profile: self.header.profile(),
            player: &self.player,
            entities: self.entities.as_source(),
            clients: self.clients.as_source(),
        }
    }
}
impl fmt::Debug for SnapshotPlan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotPlan")
            .field("profile", &self.header.profile())
            .field("entity_stats", &self.entities.stats())
            .field("client_stats", &self.clients.stats())
            .field("consumed_bytes", &self.consumed_bytes)
            .finish_non_exhaustive()
    }
}
