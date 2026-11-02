// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    gamestate::entity_baselines::{EntityBaselineDecoder, LegacyMapCenter},
    message_body::BodyReader,
};

#[derive(Clone, Copy)]
pub(crate) enum Coordinate {
    Small(u8),
    Full(u16),
}
pub(crate) fn coordinate(writer: &mut MessageWriter<'_>, value: Coordinate) {
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
pub(crate) fn header(writer: &mut MessageWriter<'_>, number: u32, count: u32, kind: u8) {
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
pub(crate) fn fixture(
    number: u32,
    coordinates: [Coordinate; 3],
    prefix: bool,
    tail: bool,
) -> Vec<u8> {
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
pub(crate) fn decoder(center: [f32; 3]) -> EntityBaselineDecoder {
    EntityBaselineDecoder::with_legacy_map_center(
        LegacyMapCenter::new(center).expect("Checked center"),
    )
}
pub(crate) fn position(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
