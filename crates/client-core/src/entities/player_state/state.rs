// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{
    EntityDecodeContext, hud::SnapshotHud, objectives::SnapshotObjectives,
    player_scalars::PlayerScalars,
};
use super::{PlayerStateError, PredictionProvenance, PreparedPlayerState, decode};
use crate::{
    BaselineProtocol,
    message_body::BodyReader,
    player_arrays::{PlayerAmmunition, PlayerStats, WeaponModels},
};
use std::fmt;

#[derive(Clone, Eq, PartialEq)]
pub(super) struct DecodedPlayerState {
    pub context: EntityDecodeContext,
    pub scalars: PlayerScalars,
    pub stats: PlayerStats,
    pub ammunition: PlayerAmmunition,
    pub objectives: SnapshotObjectives,
    pub hud: SnapshotHud,
    pub models: WeaponModels,
}
/// Owned complete live wire projection after explicit native prediction resolution.
/// This is neither typed gameplay state nor whole-snapshot/message admission.
#[derive(Clone, Eq, PartialEq)]
pub struct LivePlayerState {
    pub(super) decoded: DecodedPlayerState,
    pub(super) prediction: PredictionProvenance,
}
impl LivePlayerState {
    /// Prepares every native section while holding an exclusive reader lease.
    /// `None` supplies zero references. The returned lease owns all decoded/prior
    /// data; the reference may be dropped before prediction resolution. Dropping
    /// the lease cancels all sections and preserves cached bits and byte position.
    /// Same-profile legacy map-center changes use the new explicit context.
    ///
    /// # Errors
    /// Rejects profile mismatch, malformed fields or any truncated section without
    /// publishing reader movement. Prediction resolution occurs only at commitment.
    pub fn prepare<'reader, 'body>(
        reader: &'reader mut BodyReader<'body>,
        context: EntityDecodeContext,
        time: u32,
        reference: Option<&Self>,
    ) -> Result<PreparedPlayerState<'reader, 'body>, PlayerStateError> {
        decode::prepare(reader, context, time, reference)
    }
    /// Returns the current explicit coordinate context.
    #[must_use]
    pub fn context(&self) -> EntityDecodeContext {
        self.decoded.context
    }
    /// Returns the current explicit baseline profile.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.context().profile()
    }
    /// Borrows inert scalar words after native prediction replacement.
    #[must_use]
    pub fn scalars(&self) -> &PlayerScalars {
        &self.decoded.scalars
    }
    /// Borrows all native stat words.
    #[must_use]
    pub fn stats(&self) -> &PlayerStats {
        &self.decoded.stats
    }
    /// Borrows the wire-visible stored and clip ammunition projections.
    #[must_use]
    pub fn ammunition(&self) -> &PlayerAmmunition {
        &self.decoded.ammunition
    }
    /// Borrows all native objective slots, including inert statuses.
    #[must_use]
    pub fn objectives(&self) -> &SnapshotObjectives {
        &self.decoded.objectives
    }
    /// Borrows both complete HUD arrays, including inert tail words.
    #[must_use]
    pub fn hud(&self) -> &SnapshotHud {
        &self.decoded.hud
    }
    /// Borrows raw native per-weapon model bytes.
    #[must_use]
    pub fn models(&self) -> &WeaponModels {
        &self.decoded.models
    }
    /// Returns how the native prediction words were resolved.
    #[must_use]
    pub fn prediction(&self) -> PredictionProvenance {
        self.prediction
    }
}
impl fmt::Debug for LivePlayerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LivePlayerState")
            .field("profile", &self.profile())
            .field("prediction", &self.prediction)
            .finish_non_exhaustive()
    }
}
