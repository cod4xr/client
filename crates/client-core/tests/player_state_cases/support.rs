// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BaselineProtocol, BodyReader, EntityDecodeContext, LegacyMapCenter, LivePlayerState,
    MessageWriter, PredictionSample, PredictionWords, word,
};
use cod4xr_client_core::player_scalars::word::ALL;

pub(super) fn contexts() -> [EntityDecodeContext; 2] {
    [
        EntityDecodeContext::Cod4x21,
        EntityDecodeContext::Legacy6(LegacyMapCenter::new([0.0; 3]).expect("Checked zero center")),
    ]
}
pub(super) fn encoded(write: impl FnOnce(&mut MessageWriter<'_>)) -> Vec<u8> {
    let mut bytes = [0; 4096];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    write(&mut writer);
    writer.as_bytes().to_vec()
}
pub(super) fn empty(writer: &mut MessageWriter<'_>, transmitted: bool) {
    writer.write_bits(u32::from(transmitted), 1).expect("Mode");
    writer.write_bits(0, 8).expect("No scalar fields");
    trailing(writer);
}
pub(super) fn trailing(writer: &mut MessageWriter<'_>) {
    // Stats, stored global, eight clip groups, objectives, HUD, model flag.
    writer.write_bits(0, 13).expect("Unchanged array sections");
}
pub(super) fn short(writer: &mut MessageWriter<'_>, value: u16) {
    for byte in value.to_le_bytes() {
        writer.write_u8(byte).expect("Short byte");
    }
}
pub(super) fn mixed(
    writer: &mut MessageWriter<'_>,
    transmitted: bool,
    prior_origin: u32,
    command_time: u32,
) {
    writer.write_bits(u32::from(transmitted), 1).expect("Mode");
    writer
        .write_bits(14, 8)
        .expect("Scalar count through weapon time");
    for &key in &ALL[..14] {
        if key == word::session::COMMAND_TIME {
            writer.write_bits(3, 2).expect("Changed absolute time");
            writer.write_i32(command_time.cast_signed()).expect("Time");
        } else if key == word::motion::ORIGIN_X {
            writer.write_bits(1, 1).expect("Changed raw origin");
            writer
                .write_i32((prior_origin ^ 123.5_f32.to_bits()).cast_signed())
                .expect("Origin");
        } else if key == word::view::ANGLE_Y {
            writer.write_bits(1, 1).expect("Changed absolute angle");
            short(writer, 0x8000);
        } else if key == word::weapons::TIME {
            writer
                .write_bits(3, 2)
                .expect("Changed nonzero signed short");
            short(writer, 0xfff9);
        } else {
            writer.write_bits(0, 1).expect("Unchanged scalar");
        }
    }
    writer.write_bits(1, 1).expect("Stats changed");
    writer.write_bits(1, 5).expect("Stat zero mask");
    short(writer, 123);
    writer
        .write_bits(3, 2)
        .expect("Stored global and group zero changed");
    short(writer, 1);
    short(writer, 7);
    writer.write_bits(0, 3).expect("Other stored groups");
    writer.write_bits(0, 8).expect("Clip groups");
    writer.write_bits(1, 1).expect("Objectives changed");
    for _ in 0..16 {
        writer.write_bits(1, 3).expect("Objective status");
        writer.write_bits(0, 1).expect("Objective words unchanged");
    }
    writer.write_bits(1, 1).expect("HUD changed");
    writer.write_bits(1, 5).expect("One archival element");
    writer
        .write_bits(0, 6)
        .expect("Color-only inclusive prefix");
    writer
        .write_bits(3, 2)
        .expect("Color changed, alpha shortcut");
    writer.write_bits(0, 5).expect("No current elements");
    writer.write_bits(1, 1).expect("Model bytes changed");
    for value in 0..128_u8 {
        writer.write_u8(value ^ 0xa5).expect("Model byte");
    }
}
pub(super) fn decode(
    context: EntityDecodeContext,
    bytes: &[u8],
    prior: Option<&LivePlayerState>,
) -> LivePlayerState {
    let mut reader = BodyReader::try_new(bytes, bytes.len()).expect("Body");
    LivePlayerState::prepare(&mut reader, context, 0, prior)
        .expect("Whole state")
        .commit_transmitted()
        .expect("Transmitted state")
}
pub(super) fn sample(profile: BaselineProtocol, time: u32) -> PredictionSample {
    PredictionSample::new(
        profile,
        time,
        PredictionWords::new(
            [0x7fc0_1234, (-1.5_f32).to_bits(), 0x8000_0000],
            [10, 20, 30],
            [40, 50, 60],
            255,
            0xffff_ff80,
        ),
    )
}
pub(super) fn position(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
