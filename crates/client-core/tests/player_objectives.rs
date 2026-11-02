// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete native objective sections, scalar value reuse and atomic ownership.
#![allow(clippy::expect_used, reason = "Finite reviewed objective fixtures.")]
#[path = "objective_cases/failures.rs"]
mod failures;
#[path = "objective_cases/traces.rs"]
mod traces;
#[path = "objective_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    message_body::BodyReader,
    player_objectives::{
        OBJECTIVE_SLOTS, OBJECTIVE_WORDS, ObjectiveError, ObjectiveWord as W, SnapshotObjectives,
    },
};

const PROFILES: [BaselineProtocol; 2] = [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21];
const FIELDS: [(W, u8); 6] = [
    (W::OriginX, 0),
    (W::OriginY, 0),
    (W::OriginZ, 0),
    (W::Icon, 12),
    (W::Entity, 10),
    (W::Team, 4),
];
fn encoded(write: impl FnOnce(&mut MessageWriter<'_>)) -> Vec<u8> {
    let mut bytes = [0; 2048];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    write(&mut writer);
    writer.as_bytes().to_vec()
}
fn integer(writer: &mut MessageWriter<'_>, width: u8, prior: u32, value: u32) {
    writer.write_bits(1, 1).expect("Changed");
    writer
        .write_bits(u32::from(value != 0), 1)
        .expect("Nonzero");
    if value != 0 {
        let delta = prior ^ value;
        let residual = width % 8;
        writer
            .write_bits(delta & ((1 << residual) - 1), residual)
            .expect("Residual");
        for shift in (residual..width).step_by(8) {
            writer
                .write_u8(u8::try_from((delta >> shift) & 255).expect("Byte"))
                .expect("XOR byte");
        }
    }
}
fn raw_float(writer: &mut MessageWriter<'_>, prior: u32, value: u32) {
    writer.write_bits(7, 3).expect("Changed nonzero raw float");
    writer
        .write_i32((prior ^ value).cast_signed())
        .expect("Raw XOR");
}
fn parse(
    profile: BaselineProtocol,
    bytes: &[u8],
    prior: Option<&SnapshotObjectives>,
) -> SnapshotObjectives {
    let mut reader = BodyReader::try_new(bytes, bytes.len()).expect("Body");
    SnapshotObjectives::parse(&mut reader, profile, prior).expect("Complete objective section")
}
fn seed(profile: BaselineProtocol, value: u32) -> SnapshotObjectives {
    let bytes = encoded(|writer| {
        writer.write_bits(1, 1).expect("Section changed");
        for slot in 0..16 {
            writer.write_bits(slot % 8, 3).expect("Status");
            writer.write_bits(1, 1).expect("Entry words changed");
            for &(_, width) in &FIELDS {
                if width == 0 {
                    raw_float(writer, 0, value);
                } else {
                    integer(writer, width, 0, value & (u32::MAX >> (32 - width)));
                }
            }
        }
    });
    parse(profile, &bytes, None)
}
fn position(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
