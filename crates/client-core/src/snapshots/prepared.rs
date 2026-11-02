// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    SnapshotError, SnapshotHeader, SnapshotMessage, SnapshotPlan, transaction::Transaction,
};
use crate::{
    player_state::{PendingPlayerState, PredictionRequest, PredictionSample},
    snapshot_clients::ClientStreamStats,
    snapshot_entities::EntityStreamStats,
};
use std::fmt;

/// Exclusive whole-snapshot reader/output lease; dropping it publishes nothing.
/// No decoded player/entity/client values escape before commitment.
/// The reader remains exclusively borrowed through commitment:
///
/// ```compile_fail,E0499
/// use cod4xr_client_core::{message_body::BodyReader,
///     snapshots::{SnapshotInput, SnapshotOutput, SnapshotPlan, SnapshotError}};
/// fn reader_lease(reader: &mut BodyReader<'_>, input: SnapshotInput<'_>, output: SnapshotOutput<'_>)
///     -> Result<(), SnapshotError> {
///     let prepared = SnapshotPlan::prepare(reader, input, output)?;
///     reader.read_bits(1).unwrap();
///     prepared.commit_transmitted()?;
///     Ok(())
/// }
/// ```
/// Final output cannot be overwritten while the prepared transaction still uses it:
///
/// ```compile_fail,E0499
/// use cod4xr_client_core::{message_body::BodyReader,
///     snapshots::{SnapshotInput, SnapshotOutput, SnapshotPlan, SnapshotError}};
/// fn output_lease(reader: &mut BodyReader<'_>, input: SnapshotInput<'_>, output: SnapshotOutput<'_>)
///     -> Result<(), SnapshotError> {
///     let SnapshotOutput { entities, clients } = output;
///     let prepared = SnapshotPlan::prepare(reader, input,
///         SnapshotOutput { entities: &mut *entities, clients })?;
///     entities.fill(Default::default());
///     prepared.commit_transmitted()?;
///     Ok(())
/// }
/// ```
#[must_use = "Dropping the prepared snapshot cancels all reader and output publication."]
pub struct PreparedSnapshot<'reader, 'body, 'source, 'storage> {
    pub(super) transaction: Transaction<'reader, 'body, 'source, 'storage>,
    pub(super) player: PendingPlayerState,
}
impl<'storage> PreparedSnapshot<'_, '_, '_, 'storage> {
    /// Returns the native prediction lookup request, if needed. The host must bind
    /// its sample queue to the current world/session before supplying an exact match.
    #[must_use]
    pub fn prediction_request(&self) -> Option<PredictionRequest> {
        self.player.request()
    }
    /// Returns inert header metadata without admitting a snapshot.
    #[must_use]
    pub fn header(&self) -> SnapshotHeader {
        self.transaction.header
    }
    /// Returns the caller's world/message/server-command association.
    #[must_use]
    pub fn message(&self) -> SnapshotMessage {
        self.transaction.message
    }
    /// Returns bounded entity validation counters without state words.
    #[must_use]
    pub fn entity_stats(&self) -> EntityStreamStats {
        self.transaction.entity_stats
    }
    /// Returns bounded client-state validation counters without names/state words.
    #[must_use]
    pub fn client_stats(&self) -> ClientStreamStats {
        self.transaction.client_stats
    }
    /// Returns newly consumed physical bytes, before reader publication.
    #[must_use]
    pub fn consumed_bytes(&self) -> usize {
        self.transaction.consumed_bytes
    }
    /// Commits transmitted player state and both already-validated sparse outputs.
    ///
    /// # Errors
    /// An outstanding prediction request cancels the whole transaction. Closed
    /// immutable replay uses the exact validated source/cursor/context/budget inputs.
    pub fn commit_transmitted(self) -> Result<SnapshotPlan<'storage>, SnapshotError> {
        let player = self.player.transmitted().map_err(SnapshotError::Player)?;
        self.transaction.finish(player)
    }
    /// Commits all snapshot parts after exact prediction profile/time matching.
    ///
    /// # Errors
    /// Missing request or mismatched sample cancels both outputs and reader. No
    /// sample is retained, and no output write precedes prediction validation.
    pub fn commit_with_prediction(
        self,
        sample: &PredictionSample,
    ) -> Result<SnapshotPlan<'storage>, SnapshotError> {
        let player = self
            .player
            .predicted(sample)
            .map_err(SnapshotError::Player)?;
        self.transaction.finish(player)
    }
    /// Commits all parts using explicit native prior fallback for prediction words.
    ///
    /// # Errors
    /// Transmitted origin mode rejects fallback before any reader/output change.
    pub fn commit_native_prior_fallback(self) -> Result<SnapshotPlan<'storage>, SnapshotError> {
        let player = self
            .player
            .prior_fallback()
            .map_err(SnapshotError::Player)?;
        self.transaction.finish(player)
    }
}
impl fmt::Debug for PreparedSnapshot<'_, '_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedSnapshot")
            .field("profile", &self.header().profile())
            .field("entity_stats", &self.entity_stats())
            .field("client_stats", &self.client_stats())
            .field("needs_prediction", &self.prediction_request().is_some())
            .finish_non_exhaustive()
    }
}
