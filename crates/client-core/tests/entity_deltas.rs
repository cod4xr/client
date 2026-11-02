// SPDX-License-Identifier: AGPL-3.0-or-later
//! Individual protocol-21 delta records, reference binding and cursor/index rollback.
#![allow(clippy::expect_used, reason = "Finite checked entity-record fixtures.")]
#[path = "entity_delta_cases/failures.rs"]
mod failures;
#[path = "entity_delta_cases/traces.rs"]
mod traces;
#[path = "entity_delta_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    entity_deltas::{
        EntityDelta, EntityDeltaDecoder, EntityDeltaError, EntityState, EntityWord as W,
    },
    gamestate::entity_baselines::EntityBaselineDecoder,
    message_body::BodyReader,
};

fn decoder() -> EntityDeltaDecoder {
    EntityDeltaDecoder::new(BaselineProtocol::Cod4x21).expect("Reviewed profile")
}
fn fixture(
    number: u32,
    count: Option<u32>,
    fields: impl FnOnce(&mut MessageWriter<'_>),
) -> Vec<u8> {
    let mut bytes = [0; 512];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    writer
        .write_bits((number << 2) | 2, 12)
        .expect("Absolute index");
    writer.write_bits(0, 1).expect("Not removed");
    writer
        .write_bits(u32::from(count.is_some()), 1)
        .expect("Delta flag");
    if let Some(count) = count {
        writer.write_bits(count, 6).expect("Changed count");
    }
    fields(&mut writer);
    writer.as_bytes().to_vec()
}
fn kind(writer: &mut MessageWriter<'_>, from: u8, to: u8) {
    writer
        .write_bits(u32::from(from != to), 1)
        .expect("Type change flag");
    if from != to {
        writer
            .write_bits(u32::from(to != 0), 1)
            .expect("Type nonzero");
        if to != 0 {
            writer.write_u8(from ^ to).expect("XOR type byte");
        }
    }
}
fn reference(number: u32) -> EntityState {
    let bytes = fixture(number, Some(6), |writer| {
        kind(writer, 0, 1);
        writer.write_i32(0x7fc0_1234).expect("Raw X");
        writer
            .write_i32(1_000_000.0_f32.to_bits().cast_signed())
            .expect("Raw Y");
        writer
            .write_bits(3, 2)
            .expect("Changed nonzero signed union");
        writer.write_u8(0xfd).expect("Minus three");
        writer.write_bits(3, 2).expect("Changed nonzero angle");
        writer.write_i16(0x4000).expect("Ninety degrees");
        writer.write_bits(1, 1).expect("Changed Z");
        writer
            .write_i32(1.0_f32.to_bits().cast_signed())
            .expect("Raw Z");
    });
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Baseline body");
    let baseline = EntityBaselineDecoder::new(BaselineProtocol::Cod4x21)
        .expect("Baseline decoder")
        .parse(&mut reader)
        .expect("Owned baseline");
    EntityState::from_baseline(&baseline)
}
fn present(record: EntityDelta) -> EntityState {
    match record {
        EntityDelta::State(state) => state,
        EntityDelta::Removed { .. } => unreachable!("State fixture"),
    }
}
fn position(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
