// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::ServerPayloadDecoder;
use super::{
    PreparedSnapshotAcknowledgement, SnapshotAcknowledgementError, SnapshotDecodeStorage,
    SnapshotPayloadRequest,
};
use crate::{
    commands::LegacyClientCommands,
    snapshots::{SnapshotOutput, services::SnapshotPayloadPlan},
};

impl ServerPayloadDecoder {
    /// Validates a complete snapshot-only payload against the original outgoing
    /// command owner, holding its exclusive ACK lease for later host preflight.
    /// Supply a complete ordinary body/sequence from this decoder's original peer.
    /// No socket/clock access, peer execution or ACK/history mutation occurs here.
    /// Successful preparation produces valid cooked output; dropping it keeps that
    /// data valid while cancelling ACK release, like a prepared gamestate frame.
    ///
    /// # Errors
    /// Header/ACK, transform, decompression, binding, EOF, source/work/storage or
    /// prediction failure preserves both final output buffers and original command
    /// history. Private decode workspace may contain untrusted intermediate bytes.
    pub fn prepare_snapshot_ack<'owner, 'history, 'output>(
        &self,
        history: &'owner mut LegacyClientCommands<'history>,
        sequence: u32,
        packet: &[u8],
        storage: SnapshotDecodeStorage<'_>,
        request: SnapshotPayloadRequest<'_, '_>,
        output: SnapshotOutput<'output>,
    ) -> Result<
        PreparedSnapshotAcknowledgement<'owner, 'history, 'output>,
        SnapshotAcknowledgementError,
    > {
        let number = Self::inspect_acknowledgement(sequence, packet)
            .map_err(SnapshotAcknowledgementError::Decode)?;
        let acknowledgement = history
            .prepare_acknowledgement(number)
            .map_err(SnapshotAcknowledgementError::Acknowledgement)?;
        let SnapshotDecodeStorage { work, decoded } = storage;
        let plan = {
            let payload = self
                .decode(sequence, packet, acknowledgement.history(), work, decoded)
                .map_err(SnapshotAcknowledgementError::Decode)?;
            SnapshotPayloadPlan::parse(
                &payload,
                request.input,
                request.services,
                output,
                request.prediction,
            )
            .map_err(SnapshotAcknowledgementError::Payload)?
        };
        // No fallible operation follows cooked publication. The same exclusive
        // owner lease existed before decoding and remains held through host preflight.
        Ok(PreparedSnapshotAcknowledgement {
            acknowledgement,
            plan,
        })
    }
}
