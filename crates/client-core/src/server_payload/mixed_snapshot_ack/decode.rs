// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{ServerPayloadDecoder, SnapshotDecodeStorage};
use super::{
    MixedSnapshotAcknowledgementError, MixedSnapshotRequest, PreparedMixedSnapshotAcknowledgement,
};
use crate::{
    commands::LegacyClientCommands,
    snapshots::mixed_services::{MixedSnapshotPayloadPlan, MixedSnapshotStorage},
};

impl ServerPayloadDecoder {
    /// Holds the original outgoing command owner through complete mixed-carrier
    /// decoding and later host preflight. Packet/sequence come from the decoder's
    /// original established peer. Parsing executes no peer strings or host effects.
    /// Success produces valid cooked buffers; dropping the returned holder cancels
    /// ACK release while preserving that cooked data, without applying host history.
    ///
    /// # Errors
    /// Header/retained-key, decoding, actual binding, control/snapshot/EOF, independent
    /// budgets/capacity or prediction failure preserves all four final buffers and
    /// original outgoing ACK/key state. Private decode/control scratch is unspecified.
    pub fn prepare_mixed_snapshot_ack<'owner, 'history, 'output>(
        &self,
        history: &'owner mut LegacyClientCommands<'history>,
        sequence: u32,
        packet: &[u8],
        storage: SnapshotDecodeStorage<'_>,
        request: MixedSnapshotRequest<'_, '_>,
        output: MixedSnapshotStorage<'_, 'output>,
    ) -> Result<
        PreparedMixedSnapshotAcknowledgement<'owner, 'history, 'output>,
        MixedSnapshotAcknowledgementError,
    > {
        let number = Self::inspect_acknowledgement(sequence, packet)
            .map_err(MixedSnapshotAcknowledgementError::Decode)?;
        let acknowledgement = history
            .prepare_acknowledgement(number)
            .map_err(MixedSnapshotAcknowledgementError::Acknowledgement)?;
        let SnapshotDecodeStorage { work, decoded } = storage;
        let plan = {
            let payload = self
                .decode(sequence, packet, acknowledgement.history(), work, decoded)
                .map_err(MixedSnapshotAcknowledgementError::Decode)?;
            MixedSnapshotPayloadPlan::parse(
                &payload,
                request.input,
                request.services,
                output,
                request.prediction,
            )
            .map_err(MixedSnapshotAcknowledgementError::Payload)?
        };
        // The original exclusive key lease predates all cooked writes. Host preflight
        // follows through the returned holder, without any fallible work hereafter.
        Ok(PreparedMixedSnapshotAcknowledgement {
            acknowledgement,
            plan,
        })
    }
}
