// SPDX-License-Identifier: AGPL-3.0-or-later
//! Legacy zero-reference coordinate contexts, field layouts and atomic baseline admission.
#![allow(
    clippy::expect_used,
    reason = "Finite explicit legacy coordinate fixtures."
)]
#[path = "legacy_entity_cases/context.rs"]
mod context;
#[path = "legacy_entity_cases/failures.rs"]
mod failures;
#[path = "legacy_entity_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    gamestate::entity_baselines::{
        BaselineWord as W, EntityBaselineDecoder, EntityBaselineError, LegacyMapCenter,
    },
    message_body::BodyReader,
};

#[derive(Clone, Copy)]
enum Coordinate {
    Small(u8),
    Full(u16),
}
fn coordinate(writer: &mut MessageWriter<'_>, value: Coordinate) {
    match value {
        Coordinate::Small(value) => {
            writer.write_bits(0, 1).expect("Small coordinate");
            writer
                .write_bits(u32::from(value), 7)
                .expect("Seven coordinate bits");
        }
        Coordinate::Full(value) => {
            writer.write_bits(1, 1).expect("Full coordinate");
            writer
                .write_bits(u32::from(value), 16)
                .expect("Sixteen coordinate bits");
        }
    }
}
fn header(writer: &mut MessageWriter<'_>, number: u32, count: u32, kind: u8) {
    writer
        .write_bits((number << 2) | 2, 12)
        .expect("Absolute index");
    writer.write_bits(0, 1).expect("Not removed");
    writer.write_bits(1, 1).expect("Delta");
    writer.write_bits(count, 6).expect("Changed-field count");
    writer
        .write_bits(u32::from(kind != 0), 1)
        .expect("Type change");
    if kind != 0 {
        writer.write_bits(1, 1).expect("Type nonzero");
        writer.write_u8(kind).expect("Type byte");
    }
}
fn fixture(number: u32, coordinates: [Coordinate; 3], prefix: bool, tail: bool) -> Vec<u8> {
    let mut bytes = [0; 128];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    if prefix {
        writer.write_bits(5, 3).expect("Existing prefix");
    }
    header(&mut writer, number, 5, 0);
    writer.write_bits(0, 1).expect("Unchanged flags");
    for value in coordinates {
        writer.write_bits(1, 1).expect("Changed coordinate");
        coordinate(&mut writer, value);
    }
    if tail {
        writer.write_u8(7).expect("Following opcode");
        writer.write_bits(21, 5).expect("Following cached bits");
    }
    writer.as_bytes().to_vec()
}
fn decoder(center: [f32; 3]) -> EntityBaselineDecoder {
    EntityBaselineDecoder::with_legacy_map_center(
        LegacyMapCenter::new(center).expect("Checked center"),
    )
}
fn position(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
