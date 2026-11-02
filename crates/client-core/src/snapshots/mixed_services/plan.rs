// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{SnapshotInput, SnapshotPlan, services::SnapshotPrediction};
use super::{MixedSnapshotError, MixedSnapshotStorage, decode};
use crate::{
    server_payload::DecodedServerPayload,
    service_controls::{ClientServiceBudget, LegacyServicePlan},
};
use std::fmt;

/// Complete inert controls and one snapshot, bound to the same decoded message.
/// No command, identity history, outgoing ACK or engine state is applied.
pub struct MixedSnapshotPayloadPlan<'output> {
    pub(super) controls: LegacyServicePlan<'output>,
    pub(super) snapshot: SnapshotPlan<'output>,
}
impl<'output> MixedSnapshotPayloadPlan<'output> {
    /// Validates NOP/command/identity records on both sides of exactly one snapshot
    /// through logical EOF before publishing any final control or snapshot output.
    /// Identity updates use the reviewed protocol-21 format; legacy opcode eleven
    /// and other unsupported services are rejected without skipping their bodies.
    /// The input message's command sequence is the original host high-water mark;
    /// the snapshot association captures only newer command prefixes before it.
    ///
    /// # Errors
    /// Actual profile/message binding, malformed controls/state, missing EOF/snapshot,
    /// independent budgets/capacities or prediction failure preserves all four final
    /// buffers and their tails. Private text/identity scratch is unspecified on error.
    pub fn parse(
        payload: &DecodedServerPayload<'_, '_>,
        input: SnapshotInput<'_>,
        budget: ClientServiceBudget,
        storage: MixedSnapshotStorage<'_, 'output>,
        prediction: SnapshotPrediction<'_>,
    ) -> Result<Self, MixedSnapshotError> {
        decode::parse(payload, input, budget, storage, prediction)
    }
    /// Borrows complete normalized control records for separate owner preflight.
    #[must_use]
    pub fn controls(&self) -> &LegacyServicePlan<'output> {
        &self.controls
    }
    /// Borrows the complete coherent snapshot wire projection.
    #[must_use]
    pub fn snapshot(&self) -> &SnapshotPlan<'output> {
        &self.snapshot
    }
}
impl fmt::Debug for MixedSnapshotPayloadPlan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MixedSnapshotPayloadPlan")
            .field("profile", &self.controls.profile())
            .field("message_sequence", &self.controls.message_sequence())
            .field("records", &self.controls.record_count())
            .field("commands", &self.controls.command_count())
            .field("identity_updates", &self.controls.client_update_count())
            .field("snapshot", &self.snapshot)
            .finish_non_exhaustive()
    }
}
