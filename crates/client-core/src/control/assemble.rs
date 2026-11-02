// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    ClientControlHeader, ControlMessageError, ControlMessagePlan, MAX_CONTROL_MESSAGE_BYTES,
    MovementBatch, movement::PreparedMovement, plain,
};
use crate::commands::LegacyClientCommands;
use cod4x_protocol::{
    compression::{self, CompressionError},
    obfuscation,
};

pub(super) fn build(
    header: ClientControlHeader<'_>,
    challenge: i32,
    history: &LegacyClientCommands<'_>,
    movement: Option<MovementBatch<'_, '_>>,
) -> Result<ControlMessagePlan, ControlMessageError> {
    if header.message_acknowledge() > 0x7fff_ffff {
        return Err(ControlMessageError::Header);
    }
    let movement = movement
        .map(|batch| PreparedMovement::new(header, batch))
        .transpose()
        .map_err(ControlMessageError::Movement)?;
    let mut count = plain::select_prefix(header.encoded_len(), history, movement.as_ref())?;
    let mut staging = [0; MAX_CONTROL_MESSAGE_BYTES];
    let mut plan = ControlMessagePlan {
        bytes: [0; MAX_CONTROL_MESSAGE_BYTES],
        length: 0,
        protocol: header.requested_protocol(),
        command_count: 0,
        user_command_count: movement.as_ref().map_or(0, PreparedMovement::count),
        last_command: None,
        challenge,
    };
    loop {
        let length = plain::write(header, history, count, movement.as_ref(), &mut staging)?;
        let offset = header.encoded_len();
        plan.bytes[..offset].copy_from_slice(&staging[..offset]);
        match compression::compress(&staging[offset..length], &mut plan.bytes[offset..]) {
            Ok(compressed) => {
                plan.length = offset + compressed;
                break;
            }
            Err(CompressionError::OutputCapacity)
                if count > 1 || (count > 0 && movement.is_some()) =>
            {
                count -= 1;
            }
            Err(error) => return Err(ControlMessageError::Compression(error)),
        }
    }
    obfuscation::apply_client_payload(
        &mut plan.bytes[header.encoded_len()..plan.length],
        challenge.cast_unsigned(),
        header.server_id(),
        header.message_acknowledge(),
        header.server_command().as_bytes(),
    )
    .map_err(ControlMessageError::Obfuscation)?;
    plan.command_count = count;
    plan.last_command = (count > 0).then(|| history.acknowledged_sequence() + count);
    Ok(plan)
}
