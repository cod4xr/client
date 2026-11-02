// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{commands::PreparedClientAcknowledgement, snapshots::services::SnapshotPayloadPlan};
use std::fmt;

/// Original command-owner lease and complete cooked snapshot ready for host preflight.
/// Drop cancels ACK release; successfully cooked data remains valid and unpublished
/// to host history/engine state. Final output storage remains borrowed by this plan.
///
/// ```compile_fail,E0499
/// use cod4xr_client_core::{commands::LegacyClientCommands,
///     server_payload::{ServerPayloadDecoder, SnapshotDecodeStorage, SnapshotPayloadRequest},
///     snapshots::SnapshotOutput};
/// fn original_owner(decoder: &ServerPayloadDecoder, history: &mut LegacyClientCommands<'_>,
///     packet: &[u8], storage: SnapshotDecodeStorage<'_>, request: SnapshotPayloadRequest<'_, '_>, output: SnapshotOutput<'_>) {
///     let prepared = decoder.prepare_snapshot_ack(history, 7, packet, storage, request, output).unwrap();
///     history.commit_acknowledgement(0).unwrap();
///     prepared.commit();
/// }
/// ```
#[must_use = "Preflight host resources before committing, or drop to cancel ACK release."]
pub struct PreparedSnapshotAcknowledgement<'owner, 'history, 'output> {
    pub(super) acknowledgement: PreparedClientAcknowledgement<'owner, 'history>,
    pub(super) plan: SnapshotPayloadPlan<'output>,
}
impl<'output> PreparedSnapshotAcknowledgement<'_, '_, 'output> {
    /// Borrows complete cooked data for fallible host/session/history resource checks.
    /// This applies no simulation, content or global snapshot-history mutation.
    #[must_use]
    pub fn plan(&self) -> &SnapshotPayloadPlan<'output> {
        &self.plan
    }
    /// Commits the prechecked original-owner ACK and returns input-independent data.
    /// Exclusive ownership preserves the checked source key until infallible copying.
    /// Perform all fallible host resource checks before this consuming commit.
    #[must_use]
    pub fn commit(self) -> SnapshotPayloadPlan<'output> {
        self.acknowledgement.commit();
        self.plan
    }
}
impl fmt::Debug for PreparedSnapshotAcknowledgement<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedSnapshotAcknowledgement")
            .field("plan", &self.plan)
            .field("acknowledgement", &self.acknowledgement.sequence())
            .finish_non_exhaustive()
    }
}
