// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BaselineProtocol, BodyReader, MessageWriter, PlayerScalarPrefix, PlayerScalars, PlayerWord,
    word,
};

pub(super) fn encoded(write: impl FnOnce(&mut MessageWriter<'_>)) -> Vec<u8> {
    let mut bytes = [0; 2048];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    write(&mut writer);
    writer.as_bytes().to_vec()
}
pub(super) fn index(word: PlayerWord) -> usize {
    word::ALL
        .iter()
        .position(|&key| key == word)
        .expect("Reviewed field key")
}
pub(super) fn header(writer: &mut MessageWriter<'_>, origins: bool, count: usize) {
    writer
        .write_bits(u32::from(origins), 1)
        .expect("Origin/velocity mode");
    writer
        .write_bits(u32::try_from(count).expect("Count"), 8)
        .expect("Field count bits");
}
pub(super) fn integer(
    writer: &mut MessageWriter<'_>,
    width: u8,
    prior: u32,
    value: u32,
    outer: bool,
) {
    if outer {
        writer.write_bits(1, 1).expect("Changed");
    }
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
pub(super) fn raw_float(writer: &mut MessageWriter<'_>, prior: u32, value: u32, ordinary: bool) {
    writer.write_bits(1, 1).expect("Changed");
    if ordinary {
        writer.write_bits(3, 2).expect("Nonzero raw ordinary float");
    }
    writer
        .write_i32((prior ^ value).cast_signed())
        .expect("Raw XOR word");
}
pub(super) fn parse(
    profile: BaselineProtocol,
    time: u32,
    bytes: &[u8],
    prior: Option<&PlayerScalars>,
) -> PlayerScalarPrefix {
    let mut reader = BodyReader::try_new(bytes, bytes.len()).expect("Body");
    PlayerScalarPrefix::parse_live(&mut reader, profile, time, prior)
        .expect("Whole live scalar prefix")
}
pub(super) fn one_field(
    profile: BaselineProtocol,
    time: u32,
    origins: bool,
    prior: Option<&PlayerScalars>,
    word: PlayerWord,
    mut field: impl FnMut(&mut MessageWriter<'_>),
) -> PlayerScalarPrefix {
    let bytes = encoded(|writer| {
        header(writer, origins, index(word) + 1);
        for &key in &word::ALL[..=index(word)] {
            if key == word {
                field(writer);
            } else {
                writer
                    .write_bits(0, 1)
                    .expect("Omitted field or unconditional zero");
            }
        }
    });
    parse(profile, time, &bytes, prior)
}
pub(super) fn seed(
    profile: BaselineProtocol,
    word: PlayerWord,
    bits: i16,
    value: u32,
) -> PlayerScalars {
    one_field(profile, 0, false, None, word, |writer| {
        if bits == -88 {
            raw_float(writer, 0, value, false);
        } else if bits == 0 {
            raw_float(writer, 0, value, true);
        } else {
            integer(
                writer,
                u8::try_from(bits.unsigned_abs()).expect("Width"),
                0,
                value,
                true,
            );
        }
    })
    .into_scalars()
}
pub(super) fn position(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
