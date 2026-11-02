// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{
    SnapshotInput, prepare as snapshot,
    services::{SnapshotPayloadPlan, SnapshotPrediction},
};
use super::{MixedSnapshotError, MixedSnapshotPayloadPlan, MixedSnapshotStorage, prefix};
use crate::{
    BaselineProtocol,
    message_body::BodyReader,
    server_payload::DecodedServerPayload,
    service_controls::{
        ClientServiceBudget, ClientServiceStorage, LegacyServicePlan, ServiceControlBudget,
        ServiceControlError, client_scan::Clients, publish, scan::Layout,
    },
};
use cod4x_protocol::MAX_MESSAGE_BYTES;

pub(super) fn parse<'output>(
    payload: &DecodedServerPayload<'_, '_>,
    mut input: SnapshotInput<'_>,
    budget: ClientServiceBudget,
    storage: MixedSnapshotStorage<'_, 'output>,
    prediction: SnapshotPrediction<'_>,
) -> Result<MixedSnapshotPayloadPlan<'output>, MixedSnapshotError> {
    SnapshotPayloadPlan::check_binding(payload, input).map_err(MixedSnapshotError::Binding)?;
    let MixedSnapshotStorage {
        text_scratch,
        controls,
        snapshot: output,
    } = storage;
    let ClientServiceStorage {
        client_scratch,
        client_updates,
        command_bytes,
    } = controls;
    let limits = ServiceControlBudget {
        max_records: budget.max_records,
    };
    let mut layout = Layout::new(limits).map_err(MixedSnapshotError::Controls)?;
    let body = payload.body();
    let mut original = BodyReader::try_new(body, MAX_MESSAGE_BYTES)
        .map_err(|error| MixedSnapshotError::Controls(ServiceControlError::from(error)))?;
    let mut candidate = original.clone();
    let mut identities = match payload.profile() {
        BaselineProtocol::Cod4x21 => Clients::Accept {
            maximum: budget.max_client_updates,
            capacity: client_updates.len(),
            scratch: &mut *client_scratch,
        },
        BaselineProtocol::Legacy6 => Clients::Reject,
    };
    prefix::scan(
        &mut candidate,
        &mut layout,
        limits,
        text_scratch,
        command_bytes.len(),
        &mut identities,
    )?;
    let command_prefix = layout.newest_command(input.message.server_command_sequence());
    input.message = input.message.with_server_command_sequence(command_prefix);
    let header_start = candidate.byte_position();
    let mut prepared = snapshot::prepare_at(&mut original, candidate, input, output)
        .map_err(MixedSnapshotError::Snapshot)?;
    let snapshot_bytes = prepared.transaction.candidate.byte_position() - header_start;
    let mut tail = prepared.transaction.candidate.clone();
    loop {
        let opcode = layout
            .next(&mut tail, limits)
            .map_err(MixedSnapshotError::Controls)?;
        if layout
            .control(
                opcode,
                &mut tail,
                text_scratch,
                command_bytes.len(),
                &mut identities,
            )
            .map_err(MixedSnapshotError::Controls)?
        {
            break;
        }
    }
    prepared.transaction.candidate = tail;
    prepared.transaction.consumed_bytes = snapshot_bytes;
    let snapshot = match prediction {
        SnapshotPrediction::Transmitted => prepared.commit_transmitted(),
        SnapshotPrediction::Sample(sample) => prepared.commit_with_prediction(sample),
        SnapshotPrediction::NativePriorFallback => prepared.commit_native_prior_fallback(),
    }
    .map_err(MixedSnapshotError::Snapshot)?;
    // All records, destinations and prediction were checked before any write.
    // Closed publication repeats the exact immutable string normalization already
    // proven by the shared scan; no callback or source mutation occurs between passes.
    publish::commands(body, &layout, command_bytes).map_err(MixedSnapshotError::Controls)?;
    let count = layout.client_update_count;
    client_updates[..count].clone_from_slice(&client_scratch[..count]);
    let controls = LegacyServicePlan::from_parts(
        payload.profile(),
        payload.sequence(),
        payload.client_command_acknowledge(),
        layout,
        command_bytes,
        &client_updates[..count],
    );
    Ok(MixedSnapshotPayloadPlan { controls, snapshot })
}
