// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{MAX_PACKET_USER_COMMANDS, MovementBatch, MovementMessageError, key};
use crate::{
    control::ClientControlHeader, player_state::PredictionSample,
    user_commands::AbsoluteUserCommandPlan,
};
use cod4x_protocol::{MessageError, MessageWriter};

pub(in crate::control) struct PreparedMovement {
    commands: [Option<AbsoluteUserCommandPlan>; MAX_PACKET_USER_COMMANDS],
    count: u8,
    delta: bool,
    prediction: PredictionSample,
    bytes: usize,
    bits: usize,
}
impl PreparedMovement {
    pub fn new(
        header: ClientControlHeader<'_>,
        batch: MovementBatch<'_, '_>,
    ) -> Result<Self, MovementMessageError> {
        if batch.commands.is_empty() || batch.commands.len() > MAX_PACKET_USER_COMMANDS {
            return Err(MovementMessageError::CommandCount);
        }
        if batch.prediction.profile() != header.baseline_protocol() {
            return Err(MovementMessageError::PredictionProfile);
        }
        if batch
            .snapshot
            .is_some_and(|snapshot| snapshot.profile() != header.baseline_protocol())
        {
            return Err(MovementMessageError::SnapshotProfile);
        }
        if batch
            .snapshot
            .is_some_and(|snapshot| snapshot.message().sequence() > header.message_acknowledge())
        {
            return Err(MovementMessageError::SnapshotSequence);
        }
        let command_key = batch.checksum_feed
            ^ header.message_acknowledge()
            ^ key::hash(header.server_command().as_bytes());
        let mut result = Self {
            commands: [None; MAX_PACKET_USER_COMMANDS],
            count: u8::try_from(batch.commands.len())
                .map_err(|_| MovementMessageError::CommandCount)?,
            delta: batch.snapshot.is_some_and(|snapshot| {
                snapshot.message().sequence() == header.message_acknowledge()
            }),
            prediction: batch.prediction,
            bytes: 17,
            bits: 3,
        };
        for (slot, input) in result.commands.iter_mut().zip(batch.commands) {
            let command = AbsoluteUserCommandPlan::new(command_key, *input)
                .map_err(MovementMessageError::Command)?;
            let (bytes, bits) = command.wire_size();
            result.bytes += bytes;
            result.bits += bits;
            *slot = Some(command);
        }
        Ok(result)
    }
    pub fn count(&self) -> u8 {
        self.count
    }
    pub fn wire_size(&self) -> (usize, usize) {
        (self.bytes, self.bits)
    }
    pub fn write(&self, writer: &mut MessageWriter<'_>) -> Result<(), MessageError> {
        writer.write_bits(u32::from(!self.delta), 3)?;
        writer.write_u8(self.count)?;
        for command in self.commands[..usize::from(self.count)].iter().flatten() {
            command.write(writer)?;
        }
        for word in self.prediction.words().origin() {
            writer.write_bytes(&word.to_le_bytes())?;
        }
        writer.write_bytes(&self.prediction.command_time().to_le_bytes())
    }
}
