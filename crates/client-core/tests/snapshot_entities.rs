// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native entity stream framing, sparse merging and whole-output publication.
#![allow(
    clippy::expect_used,
    reason = "Finite checked native entity stream fixtures."
)]
#[path = "snapshot_entity_cases/bounds.rs"]
mod bounds;
#[path = "snapshot_entity_cases/ownership.rs"]
mod ownership;
#[path = "snapshot_entity_cases/traces.rs"]
mod traces;
#[path = "snapshot_entity_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    entities::{EntityBaselineDecoder, EntityDecodeContext, LegacyMapCenter},
    entity_deltas::{EntityState, EntityWord as W},
    message_body::BodyReader,
    snapshot_entities::{
        EntityStateSource, EntityStreamBudget, EntityStreamError, EntityStreamSources,
        SnapshotEntityPlan, SnapshotEntitySlot,
    },
};

const BUDGET: EntityStreamBudget = EntityStreamBudget {
    max_wire_records: 32,
    max_previous_entities: 32,
    max_baselines: 32,
    max_entities: 32,
};
#[derive(Clone, Copy)]
enum Update {
    Retain(u32),
    Remove(u32),
    Kind(u32, u8, u8),
}
fn index(writer: &mut MessageWriter<'_>, number: u32) {
    writer
        .write_bits((number << 2) | 2, 12)
        .expect("Absolute native index");
}
fn kind(writer: &mut MessageWriter<'_>, from: u8, target: u8) {
    writer
        .write_bits(u32::from(from != target), 1)
        .expect("Type changed");
    if from != target {
        writer
            .write_bits(u32::from(target != 0), 1)
            .expect("Nonzero target type");
        if target != 0 {
            writer.write_u8(from ^ target).expect("XOR type");
        }
    }
}
fn write_update(writer: &mut MessageWriter<'_>, update: Update) {
    let number = match update {
        Update::Retain(number) | Update::Remove(number) | Update::Kind(number, _, _) => number,
    };
    index(writer, number);
    writer
        .write_bits(u32::from(matches!(update, Update::Remove(_))), 1)
        .expect("Removal flag");
    if matches!(update, Update::Remove(_)) {
        return;
    }
    writer
        .write_bits(u32::from(matches!(update, Update::Kind(..))), 1)
        .expect("Delta flag");
    if let Update::Kind(_, from, target) = update {
        writer.write_bits(1, 6).expect("Changed type prefix");
        kind(writer, from, target);
    }
}
fn fixture(updates: &[Update], prefix: bool, tail: bool) -> Vec<u8> {
    let mut bytes = [0; 4096];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    if prefix {
        writer.write_bits(5, 3).expect("Existing cached prefix");
    }
    for &update in updates {
        write_update(&mut writer, update);
    }
    index(&mut writer, 1023);
    if tail {
        writer.write_u8(7).expect("Following byte");
        writer.write_bits(21, 5).expect("Following cached bits");
    }
    writer.as_bytes().to_vec()
}
fn state(number: u32, value: u8) -> EntityState {
    let mut bytes = [0; 32];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    write_update(&mut writer, Update::Kind(number, 0, value));
    let mut reader =
        BodyReader::try_new(writer.as_bytes(), writer.as_bytes().len()).expect("Baseline");
    let baseline = EntityBaselineDecoder::new(BaselineProtocol::Cod4x21)
        .expect("Profile")
        .parse(&mut reader)
        .expect("Complete baseline");
    EntityState::from_baseline(&baseline)
}
fn sources<'source>(
    previous: &'source [EntityState],
    baselines: &'source [EntityState],
) -> EntityStreamSources<'source> {
    EntityStreamSources {
        previous: EntityStateSource::from_states(previous),
        baselines: EntityStateSource::from_states(baselines),
    }
}
fn dirty(length: usize) -> Vec<SnapshotEntitySlot> {
    let bytes = fixture(&[Update::Kind(7, 0, 7)], false, false);
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Seed");
    let mut slots = [SnapshotEntitySlot::default()];
    let _ = SnapshotEntityPlan::parse(
        &mut reader,
        EntityDecodeContext::Cod4x21,
        0,
        sources(&[], &[]),
        BUDGET,
        &mut slots,
    )
    .expect("Dirty seed");
    vec![slots[0].clone(); length]
}
fn values(plan: &SnapshotEntityPlan<'_>) -> Vec<(u32, u32)> {
    plan.entities()
        .map(|entity| (entity.number(), entity.word(W::Kind)))
        .collect()
}
fn position(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
