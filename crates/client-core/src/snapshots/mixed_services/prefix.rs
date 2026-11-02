// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{MixedSnapshotError, MixedSnapshotStorage};
use crate::{
    BaselineProtocol,
    message_body::BodyReader,
    server_payload::DecodedServerPayload,
    service_controls::{
        ClientServiceBudget, ServiceControlBudget, ServiceControlError, client_scan::Clients,
        scan::Layout,
    },
    snapshots::{
        SNAPSHOT_OPCODE, SnapshotError, SnapshotHeader, SnapshotInput,
        services::SnapshotPayloadPlan,
    },
};
use cod4x_protocol::MAX_MESSAGE_BYTES;

pub(super) fn scan(
    reader: &mut BodyReader<'_>,
    layout: &mut Layout,
    limits: ServiceControlBudget,
    text: &mut [u8],
    command_capacity: usize,
    identities: &mut Clients<'_>,
) -> Result<(), MixedSnapshotError> {
    loop {
        let opcode = layout
            .next(reader, limits)
            .map_err(MixedSnapshotError::Controls)?;
        if opcode == SNAPSHOT_OPCODE {
            return Ok(());
        }
        if layout
            .control(opcode, reader, text, command_capacity, identities)
            .map_err(MixedSnapshotError::Controls)?
        {
            return Err(MixedSnapshotError::MissingSnapshot);
        }
    }
}

// This private inspection grants no publication or EOF/ACK authority. Full parsing
// repeats the same immutable prefix and checks the actual header/reference again.
pub(crate) fn inspect_header(
    payload: &DecodedServerPayload<'_, '_>,
    input: SnapshotInput<'_>,
    budget: ClientServiceBudget,
    output: &mut MixedSnapshotStorage<'_, '_>,
) -> Result<SnapshotHeader, MixedSnapshotError> {
    SnapshotPayloadPlan::check_binding(payload, input).map_err(MixedSnapshotError::Binding)?;
    let limits = ServiceControlBudget {
        max_records: budget.max_records,
    };
    let mut layout = Layout::new(limits).map_err(MixedSnapshotError::Controls)?;
    let mut reader = BodyReader::try_new(payload.body(), MAX_MESSAGE_BYTES)
        .map_err(|error| MixedSnapshotError::Controls(ServiceControlError::from(error)))?;
    let mut identities = match payload.profile() {
        BaselineProtocol::Cod4x21 => Clients::Accept {
            maximum: budget.max_client_updates,
            capacity: output.controls.client_updates.len(),
            scratch: &mut *output.controls.client_scratch,
        },
        BaselineProtocol::Legacy6 => Clients::Reject,
    };
    scan(
        &mut reader,
        &mut layout,
        limits,
        output.text_scratch,
        output.controls.command_bytes.len(),
        &mut identities,
    )?;
    SnapshotHeader::parse(&mut reader, payload.profile())
        .map_err(|error| MixedSnapshotError::Snapshot(SnapshotError::Body(error)))
}
