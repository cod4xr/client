// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BaselineProtocol, BodyReader, ClientStateWord, ClientStreamBudget, EntityDecodeContext,
    EntityStateSource, EntityStreamBudget, EntityWord, LegacyMapCenter, MessageWriter,
    PredictionSample, PredictionWords, SnapshotBaselines, SnapshotBudget, SnapshotClientSlot,
    SnapshotEntitySlot, SnapshotInput, SnapshotMessage, SnapshotOutput, SnapshotPlan,
    SnapshotSource,
};

pub(super) const BUDGET: SnapshotBudget = SnapshotBudget {
    entities: EntityStreamBudget {
        max_wire_records: 1025,
        max_previous_entities: 1023,
        max_baselines: 1024,
        max_entities: 1023,
    },
    clients: ClientStreamBudget {
        max_wire_records: 65,
        max_previous_clients: 64,
        max_clients: 64,
    },
};
#[derive(Clone, Copy)]
pub(super) enum Change {
    Keep(u32),
    Remove(u32),
    Set(u32, u16, u16),
}
impl Change {
    fn number(self) -> u32 {
        match self {
            Self::Keep(n) | Self::Remove(n) | Self::Set(n, _, _) => n,
        }
    }
}
pub(super) fn contexts() -> [EntityDecodeContext; 2] {
    [
        EntityDecodeContext::Cod4x21,
        EntityDecodeContext::Legacy6(LegacyMapCenter::new([1.0, -2.0, 3.0]).expect("Center")),
    ]
}
pub(super) fn input(
    context: EntityDecodeContext,
    sequence: u32,
    previous: Option<SnapshotSource<'_>>,
) -> SnapshotInput<'_> {
    SnapshotInput {
        context,
        message: SnapshotMessage::new(7, sequence, 0xffff_fffe).expect("Message"),
        previous,
        baselines: SnapshotBaselines::new(
            7,
            context.profile(),
            EntityStateSource::from_states(&[]),
        ),
        budget: BUDGET,
    }
}
pub(super) fn output<'storage>(
    entities: &'storage mut [SnapshotEntitySlot],
    clients: &'storage mut [SnapshotClientSlot],
) -> SnapshotOutput<'storage> {
    SnapshotOutput { entities, clients }
}
pub(super) fn encoded(write: impl FnOnce(&mut MessageWriter<'_>)) -> Vec<u8> {
    let mut bytes = [0; 4096];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    write(&mut writer);
    writer.as_bytes().to_vec()
}
pub(super) fn prefix(writer: &mut MessageWriter<'_>, distance: u8, transmitted: bool) {
    writer.write_i32(0x7654_3210).expect("Time");
    writer.write_u8(distance).expect("Distance");
    writer.write_u8(0xa5).expect("Flags");
    writer
        .write_bits(u32::from(transmitted), 1)
        .expect("Origin mode");
    writer.write_bits(0, 8).expect("Scalar count");
    writer.write_bits(0, 13).expect("Unchanged arrays");
}
pub(super) fn entity(writer: &mut MessageWriter<'_>, change: Change) {
    writer
        .write_bits((change.number() << 2) | 2, 12)
        .expect("Entity absolute index");
    writer
        .write_bits(u32::from(matches!(change, Change::Remove(_))), 1)
        .expect("Removal");
    if matches!(change, Change::Remove(_)) {
        return;
    }
    writer
        .write_bits(u32::from(matches!(change, Change::Set(..))), 1)
        .expect("Delta");
    if let Change::Set(_, from, to) = change {
        writer.write_bits(1, 6).expect("Type prefix");
        writer
            .write_bits(u32::from(from != to), 1)
            .expect("Changed type");
        if from != to {
            writer.write_bits(u32::from(to != 0), 1).expect("Nonzero");
            if to != 0 {
                writer
                    .write_u8(u8::try_from(from ^ to).expect("Kind byte"))
                    .expect("Kind XOR");
            }
        }
    }
}
pub(super) fn client(writer: &mut MessageWriter<'_>, change: Change) {
    writer
        .write_bits(1 | (change.number() << 2), 8)
        .expect("More and absolute client index");
    writer
        .write_bits(u32::from(matches!(change, Change::Remove(_))), 1)
        .expect("Removal");
    if matches!(change, Change::Remove(_)) {
        return;
    }
    writer
        .write_bits(u32::from(matches!(change, Change::Set(..))), 1)
        .expect("Delta");
    if let Change::Set(_, from, to) = change {
        writer.write_bits(1, 5).expect("Model prefix");
        writer.write_bits(1, 1).expect("Model changed");
        writer.write_bits(u32::from(to != 0), 1).expect("Nonzero");
        if to != 0 {
            writer
                .write_bits(u32::from((from ^ to) & 1), 1)
                .expect("Residual");
            writer
                .write_u8(u8::try_from((from ^ to) >> 1).expect("Byte"))
                .expect("Model XOR");
        }
    }
}
pub(super) fn empty_tail(writer: &mut MessageWriter<'_>) {
    writer.write_bits(0xffe, 12).expect("Entity terminator");
    writer.write_bits(0, 1).expect("Client end");
}
pub(super) fn body(
    distance: u8,
    transmitted: bool,
    entities: &[Change],
    clients: &[Change],
) -> Vec<u8> {
    encoded(|writer| {
        prefix(writer, distance, transmitted);
        for &change in entities {
            entity(writer, change);
        }
        writer.write_bits(0xffe, 12).expect("Entity terminator");
        for &change in clients {
            client(writer, change);
        }
        writer.write_bits(0, 1).expect("Client end");
    })
}
pub(super) fn dirty() -> (Vec<SnapshotEntitySlot>, Vec<SnapshotClientSlot>) {
    let bytes = body(0, true, &[Change::Set(4, 0, 7)], &[Change::Set(5, 0, 511)]);
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Seed body");
    let mut entities = [SnapshotEntitySlot::default()];
    let mut clients = [SnapshotClientSlot::default()];
    let _ = SnapshotPlan::prepare(
        &mut reader,
        input(EntityDecodeContext::Cod4x21, 1, None),
        output(&mut entities, &mut clients),
    )
    .expect("Seed")
    .commit_transmitted()
    .expect("Seed commit");
    (vec![entities[0].clone(); 4], vec![clients[0].clone(); 4])
}
pub(super) fn sample(profile: BaselineProtocol) -> PredictionSample {
    PredictionSample::new(
        profile,
        0,
        PredictionWords::new([1, 2, 3], [4, 5, 6], [7, 8, 9], 255, u32::MAX),
    )
}
type StateValues = Vec<(u32, u32)>;
pub(super) fn values(plan: &SnapshotPlan<'_>) -> (StateValues, StateValues) {
    (
        plan.entities()
            .entities()
            .map(|state| (state.number(), state.word(EntityWord::Kind)))
            .collect(),
        plan.clients()
            .clients()
            .map(|state| (state.number(), state.word(ClientStateWord::Model)))
            .collect(),
    )
}
pub(super) fn position(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
