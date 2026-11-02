// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BodyReader, EntityDecodeContext, FIELDS, MessageWriter, SnapshotHud, W};

pub(super) fn encoded(write: impl FnOnce(&mut MessageWriter<'_>)) -> Vec<u8> {
    let mut bytes = vec![0; 32768];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    write(&mut writer);
    writer.as_bytes().to_vec()
}
pub(super) fn array(
    writer: &mut MessageWriter<'_>,
    count: u32,
    last: u32,
    mut field: impl FnMut(usize, usize, &mut MessageWriter<'_>),
) {
    writer.write_bits(count, 5).expect("Array count");
    for slot in 0..count {
        writer.write_bits(last, 6).expect("Inclusive last field");
        for index in 0..=last {
            field(
                usize::try_from(slot).expect("Slot"),
                usize::try_from(index).expect("Field"),
                writer,
            );
        }
    }
}
pub(super) fn integer(writer: &mut MessageWriter<'_>, width: u8, prior: u32, value: u32) {
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
pub(super) fn raw_float(writer: &mut MessageWriter<'_>, prior: u32, value: u32) {
    // Both ordinary and HUD compact float rules accept this explicit raw form.
    writer.write_bits(7, 3).expect("Changed nonzero raw float");
    writer
        .write_i32((prior ^ value).cast_signed())
        .expect("Raw XOR word");
}
pub(super) fn parse(
    context: EntityDecodeContext,
    time: u32,
    bytes: &[u8],
    prior: Option<&SnapshotHud>,
) -> SnapshotHud {
    let mut reader = BodyReader::try_new(bytes, bytes.len()).expect("Body");
    SnapshotHud::parse(&mut reader, context, time, prior).expect("Whole HUD section")
}
pub(super) fn one_field(
    context: EntityDecodeContext,
    time: u32,
    prior: Option<&SnapshotHud>,
    word: W,
    mut field: impl FnMut(&mut MessageWriter<'_>),
) -> SnapshotHud {
    let bytes = encoded(|writer| {
        writer.write_bits(1, 1).expect("HUD changed");
        array(writer, 1, 39, |_, index, writer| {
            if FIELDS[index].0 == word {
                field(writer);
            } else {
                writer.write_bits(0, 1).expect("Unchanged field");
            }
        });
        writer.write_bits(0, 5).expect("No current prefix");
    });
    parse(context, time, &bytes, prior)
}
pub(super) fn seed(context: EntityDecodeContext, kinds: &[u32], value: u32) -> SnapshotHud {
    let bytes = encoded(|writer| {
        writer.write_bits(1, 1).expect("HUD changed");
        array(
            writer,
            u32::try_from(kinds.len()).expect("Count"),
            39,
            |slot, index, writer| match FIELDS[index].0 {
                W::Kind => integer(writer, 4, 0, kinds[slot]),
                W::FromX | W::Value => raw_float(writer, 0, value),
                _ => writer.write_bits(0, 1).expect("Unchanged field"),
            },
        );
        writer.write_bits(0, 5).expect("No current prefix");
    });
    parse(context, 0, &bytes, None)
}
pub(super) fn position(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
