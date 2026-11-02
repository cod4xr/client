// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    ClientControlHeader, ControlMessageError, MAX_CONTROL_MESSAGE_BYTES, movement::PreparedMovement,
};
use crate::commands::LegacyClientCommands;
use cod4x_protocol::{MessageError, MessageStringLimit, MessageWriter};

pub(super) fn select_prefix(
    header_bytes: usize,
    history: &LegacyClientCommands<'_>,
    movement: Option<&PreparedMovement>,
) -> Result<u32, ControlMessageError> {
    let (move_bytes, move_bits) = movement.map_or((0, 0), PreparedMovement::wire_size);
    let minimum = header_bytes + move_bytes + (move_bits + 3).div_ceil(8);
    if minimum > MAX_CONTROL_MESSAGE_BYTES {
        return Err(ControlMessageError::Message(MessageError::Capacity {
            needed: minimum,
            remaining: MAX_CONTROL_MESSAGE_BYTES,
        }));
    }
    let mut count = 0_u32;
    let mut fields = 0;
    for sequence in history.acknowledged_sequence() + 1..=history.latest_sequence() {
        let command = history
            .command(sequence)
            .map_err(ControlMessageError::History)?;
        let candidate_fields = fields + 4 + command.as_bytes().len() + 1;
        let candidate_bits =
            move_bits + 3 * (usize::try_from(count).map_err(|_| ControlMessageError::Header)? + 2);
        if header_bytes + move_bytes + candidate_fields + candidate_bits.div_ceil(8)
            > MAX_CONTROL_MESSAGE_BYTES
        {
            break;
        }
        fields = candidate_fields;
        count += 1;
    }
    Ok(count)
}
pub(super) fn write(
    header: ClientControlHeader<'_>,
    history: &LegacyClientCommands<'_>,
    count: u32,
    movement: Option<&PreparedMovement>,
    output: &mut [u8],
) -> Result<usize, ControlMessageError> {
    let mut writer = MessageWriter::try_new(output).map_err(ControlMessageError::Message)?;
    header
        .write(&mut writer)
        .map_err(ControlMessageError::Message)?;
    for sequence in history.acknowledged_sequence() + 1..=history.acknowledged_sequence() + count {
        let command = history
            .command(sequence)
            .map_err(ControlMessageError::History)?;
        writer
            .write_bits(2, 3)
            .map_err(ControlMessageError::Message)?;
        writer
            .write_bytes(&sequence.to_le_bytes())
            .map_err(ControlMessageError::Message)?;
        writer
            .write_legacy_string(command.as_bytes(), MessageStringLimit::Normal)
            .map_err(ControlMessageError::Message)?;
    }
    if let Some(movement) = movement {
        movement
            .write(&mut writer)
            .map_err(ControlMessageError::Message)?;
    }
    writer
        .write_bits(3, 3)
        .map_err(ControlMessageError::Message)?;
    Ok(writer.len())
}
