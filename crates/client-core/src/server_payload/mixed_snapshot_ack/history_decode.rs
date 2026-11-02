// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{ServerPayloadDecoder, SnapshotDecodeStorage};
use super::{
    MixedSnapshotAcknowledgementError, MixedSnapshotHistoryRequest,
    PreparedMixedSnapshotAcknowledgement,
};
use crate::{
    commands::LegacyClientCommands,
    snapshots::{
        SnapshotInput,
        mixed_services::{MixedSnapshotPayloadPlan, MixedSnapshotStorage, inspect_header},
    },
};

impl ServerPayloadDecoder {
    /// Selects an exact retained prior from this packet's actual snapshot header.
    /// The original outgoing key is held before the single transform/decompression.
    /// A private shared prefix scan locates the header without final output writes;
    /// complete parsing repeats prefix validation and checks all state, tail and EOF.
    /// The returned holder retains no packet, workspace or prior-history borrow.
    ///
    /// # Errors
    /// Header/ACK/decode, actual binding, malformed prefix/header, exact history
    /// resolution, full carrier/budgets or prediction failure preserves all four
    /// final buffers, the original outgoing key/ACK and every retained snapshot.
    /// Private decode/control scratch remains unspecified. No fallback is inferred.
    pub fn prepare_mixed_snapshot_history_ack<'owner, 'history, 'output>(
        &self,
        outgoing: &'owner mut LegacyClientCommands<'history>,
        sequence: u32,
        packet: &[u8],
        storage: SnapshotDecodeStorage<'_>,
        request: MixedSnapshotHistoryRequest<'_, '_, '_, '_>,
        mut output: MixedSnapshotStorage<'_, 'output>,
    ) -> Result<
        PreparedMixedSnapshotAcknowledgement<'owner, 'history, 'output>,
        MixedSnapshotAcknowledgementError,
    > {
        let number = Self::inspect_acknowledgement(sequence, packet)
            .map_err(MixedSnapshotAcknowledgementError::Decode)?;
        let acknowledgement = outgoing
            .prepare_acknowledgement(number)
            .map_err(MixedSnapshotAcknowledgementError::Acknowledgement)?;
        let SnapshotDecodeStorage { work, decoded } = storage;
        let plan = {
            let payload = self
                .decode(sequence, packet, acknowledgement.history(), work, decoded)
                .map_err(MixedSnapshotAcknowledgementError::Decode)?;
            let mut input = SnapshotInput {
                context: request.context,
                message: request.message,
                previous: None,
                baselines: request.baselines,
                budget: request.budget,
            };
            let header = inspect_header(&payload, input, request.services, &mut output)
                .map_err(MixedSnapshotAcknowledgementError::Payload)?;
            input.previous = request
                .history
                .reference(request.message, header)
                .map_err(MixedSnapshotAcknowledgementError::History)?;
            MixedSnapshotPayloadPlan::parse(
                &payload,
                input,
                request.services,
                output,
                request.prediction,
            )
            .map_err(MixedSnapshotAcknowledgementError::Payload)?
        };
        Ok(PreparedMixedSnapshotAcknowledgement {
            acknowledgement,
            plan,
        })
    }
}
