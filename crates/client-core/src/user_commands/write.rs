// SPDX-License-Identifier: AGPL-3.0-or-later
use super::UserCommandInput;
use cod4x_protocol::{MessageError, MessageWriter};

pub(super) fn write(
    writer: &mut MessageWriter<'_>,
    key: u32,
    input: UserCommandInput,
) -> Result<(), MessageError> {
    writer.write_bits(0, 1)?;
    writer.write_i32(input.server_time)?;
    writer.write_bits(key ^ 1, 1)?;
    writer.write_bits(key ^ 1, 1)?;
    writer.write_bits(key ^ input.buttons, 1)?;
    short(writer, key, input.angles[0])?;
    short(writer, key, input.angles[1])?;
    let movement = input.movement[0].mask() | (input.movement[1].mask() << 2);
    bits(writer, key, movement, 4)?;
    // Native full form rekeys only after attack, pitch/yaw and movement.
    let key = key ^ input.server_time.cast_unsigned();
    short(writer, key, input.angles[2])?;
    bits(writer, key, input.buttons >> 1, 20)?;
    bits(writer, key, u32::from(input.weapon), 7)?;
    bits(writer, key, u32::from(input.offhand), 7)?;
    if let Some(location) = input.selected_location {
        for value in location {
            writer.write_bits(1, 1)?;
            writer.write_u8(value ^ key.to_le_bytes()[0])?;
        }
    }
    if let Some(melee) = input.melee {
        short(writer, key, melee.yaw)?;
        // The native distance uses the bit cursor; selected-location uses bytes.
        bits(writer, key, u32::from(melee.distance), 8)?;
    }
    Ok(())
}
fn short(writer: &mut MessageWriter<'_>, key: u32, value: u16) -> Result<(), MessageError> {
    writer.write_bits(1, 1)?;
    let key = key.to_le_bytes();
    let value = value.to_le_bytes();
    writer.write_i16(i16::from_le_bytes([value[0] ^ key[0], value[1] ^ key[1]]))
}
fn bits(
    writer: &mut MessageWriter<'_>,
    key: u32,
    value: u32,
    width: u8,
) -> Result<(), MessageError> {
    writer.write_bits(1, 1)?;
    writer.write_bits(value ^ key, width)
}
