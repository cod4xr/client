// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::snapshots::{
    BodyReader, Change, EntityDecodeContext, SnapshotClientSlot, SnapshotEntitySlot, SnapshotInput,
    SnapshotMessage, SnapshotPlan, body, input, output,
};
use cod4xr_client_core::{
    snapshot_entities::EntityStateSource,
    snapshots::{SnapshotBaselines, SnapshotHeader},
};

pub(super) fn full<'output>(
    context: EntityDecodeContext,
    epoch: u64,
    sequence: u32,
    entities: &'output mut [SnapshotEntitySlot],
    clients: &'output mut [SnapshotClientSlot],
) -> SnapshotPlan<'output> {
    let bytes = body(
        0,
        true,
        &[Change::Set(1, 0, 13)],
        &[Change::Set(63, 0, 511)],
    );
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Full wire snapshot");
    SnapshotPlan::prepare(
        &mut reader,
        selected(context, epoch, sequence),
        output(entities, clients),
    )
    .expect("Complete snapshot")
    .commit_transmitted()
    .expect("Transmitted state")
}
pub(super) fn selected(
    context: EntityDecodeContext,
    epoch: u64,
    sequence: u32,
) -> SnapshotInput<'static> {
    let mut selected = input(context, sequence, None);
    selected.message = SnapshotMessage::new(epoch, sequence, 3).expect("Host association");
    selected.baselines = SnapshotBaselines::new(
        epoch,
        context.profile(),
        EntityStateSource::from_states(&[]),
    );
    selected
}
pub(super) fn header(context: EntityDecodeContext, distance: u8) -> SnapshotHeader {
    let bytes = [123, 0, 0, 0, distance, 0xa5];
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Header bytes");
    SnapshotHeader::parse(&mut reader, context.profile()).expect("Native header")
}
