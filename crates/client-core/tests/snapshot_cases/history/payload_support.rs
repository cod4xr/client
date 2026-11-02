// SPDX-License-Identifier: AGPL-3.0-or-later
use super::support::full;
use crate::{
    service_support::Fixture,
    snapshots::{
        EntityDecodeContext, SnapshotClientSlot, SnapshotEntitySlot,
        mixed_support::{Finals, LIMITS, selected},
    },
};
use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4xr_client_core::{
    commands::LegacyClientCommands,
    server_payload::{
        MixedSnapshotAcknowledgementError, MixedSnapshotHistoryRequest,
        PreparedMixedSnapshotAcknowledgement, ServerPayloadDecoder, SnapshotDecodeStorage,
    },
    service_controls::ClientDataUpdateSlot,
    snapshots::{history::SnapshotHistory, services::SnapshotPrediction},
};

pub(in crate::snapshots) fn outgoing(bytes: &mut [u8]) -> LegacyClientCommands<'_> {
    let mut owner = LegacyClientCommands::new(bytes).expect("Original outgoing owner");
    owner.enqueue(b"PrivateKey").expect("Retained key");
    owner
        .mark_transmitted(1)
        .expect("Synthetic host transmission");
    owner
}
pub(in crate::snapshots) fn seed(
    history: &mut SnapshotHistory<'_, '_>,
    context: EntityDecodeContext,
    sequence: u32,
) {
    let mut entities = [SnapshotEntitySlot::default()];
    let mut clients = [SnapshotClientSlot::default()];
    let plan = full(
        context,
        history.epoch(),
        sequence,
        &mut entities,
        &mut clients,
    );
    let _ = history
        .prepare(&plan)
        .expect("Seed coherent retained state")
        .commit();
}
pub(in crate::snapshots) fn request<'source, 'slots, 'storage>(
    context: EntityDecodeContext,
    history: &'source SnapshotHistory<'slots, 'storage>,
) -> MixedSnapshotHistoryRequest<'source, 'slots, 'storage, 'static> {
    let current = selected(context, 7);
    MixedSnapshotHistoryRequest {
        context,
        message: current.message,
        baselines: current.baselines,
        budget: current.budget,
        services: LIMITS,
        prediction: SnapshotPrediction::Transmitted,
        history,
    }
}
pub(in crate::snapshots) struct Attempt {
    fixture: Fixture,
    payload_codec: ServerPayloadDecoder,
    work: Vec<u8>,
    decoded: Vec<u8>,
    text: [u8; 128],
    updates: Vec<ClientDataUpdateSlot>,
}
impl Attempt {
    pub fn new(source: &[u8], context: EntityDecodeContext) -> Self {
        Self {
            fixture: Fixture::new(source, 1, b"PrivateKey"),
            payload_codec: ServerPayloadDecoder::new(context.profile(), 0x1234_5678),
            work: vec![0; MAX_MESSAGE_BYTES],
            decoded: vec![0; MAX_MESSAGE_BYTES],
            text: [0; 128],
            updates: vec![ClientDataUpdateSlot::default(); 4],
        }
    }
    pub fn prepare<'owner, 'history, 'output>(
        &mut self,
        outgoing: &'owner mut LegacyClientCommands<'history>,
        request: MixedSnapshotHistoryRequest<'_, '_, '_, '_>,
        finals: &'output mut Finals,
    ) -> Result<
        PreparedMixedSnapshotAcknowledgement<'owner, 'history, 'output>,
        MixedSnapshotAcknowledgementError,
    > {
        self.payload_codec.prepare_mixed_snapshot_history_ack(
            outgoing,
            7,
            self.fixture.packet(),
            SnapshotDecodeStorage {
                work: &mut self.work,
                decoded: &mut self.decoded,
            },
            request,
            finals.storage(&mut self.text, &mut self.updates),
        )
    }
}
