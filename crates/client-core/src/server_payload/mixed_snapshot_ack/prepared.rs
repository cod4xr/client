// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    commands::PreparedClientAcknowledgement, snapshots::mixed_services::MixedSnapshotPayloadPlan,
};
use std::fmt;

/// Original outgoing-owner lease and complete cooked mixed carrier for host preflight.
/// Drop cancels ACK release; cooked buffers remain valid without host-history effects.
///
/// ```compile_fail,E0499
/// use cod4xr_client_core::{commands::LegacyClientCommands,
///     server_payload::{ServerPayloadDecoder, SnapshotDecodeStorage, MixedSnapshotRequest},
///     snapshots::mixed_services::MixedSnapshotStorage};
/// fn original_owner(decoder: &ServerPayloadDecoder, history: &mut LegacyClientCommands<'_>,
///     packet: &[u8], storage: SnapshotDecodeStorage<'_>, request: MixedSnapshotRequest<'_, '_>, output: MixedSnapshotStorage<'_, '_>) {
///     let prepared = decoder.prepare_mixed_snapshot_ack(history, 7, packet, storage, request, output).unwrap();
///     history.commit_acknowledgement(0).unwrap();
///     prepared.commit();
/// }
/// ```
#[must_use = "Preflight all host resources before committing, or drop to cancel ACK release."]
pub struct PreparedMixedSnapshotAcknowledgement<'owner, 'history, 'output> {
    pub(super) acknowledgement: PreparedClientAcknowledgement<'owner, 'history>,
    pub(super) plan: MixedSnapshotPayloadPlan<'output>,
}
impl<'output> PreparedMixedSnapshotAcknowledgement<'_, '_, 'output> {
    /// Borrows complete inert controls/snapshot data for command, identity, content
    /// and host-history resource preflight. This applies none of those effects.
    #[must_use]
    pub fn plan(&self) -> &MixedSnapshotPayloadPlan<'output> {
        &self.plan
    }
    /// Commits the checked original outgoing ACK and returns the complete cooked data.
    /// Perform every fallible host preflight first. Commit infallible child command/
    /// identity preparations borrowing this plan before consuming this holder.
    #[must_use]
    pub fn commit(self) -> MixedSnapshotPayloadPlan<'output> {
        self.acknowledgement.commit();
        self.plan
    }
}
impl fmt::Debug for PreparedMixedSnapshotAcknowledgement<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedMixedSnapshotAcknowledgement")
            .field("plan", &self.plan)
            .field("acknowledgement", &self.acknowledgement.sequence())
            .finish_non_exhaustive()
    }
}
