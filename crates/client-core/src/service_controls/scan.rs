// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{MAX_BASELINE_SERVER_COMMANDS, ServiceControlBudget, ServiceControlError};
use cod4x_protocol::{MessageReader, MessageStringLimit};

#[derive(Clone, Copy, Default)]
pub(super) struct CommandDescriptor {
    pub(super) sequence: u32,
    pub(super) source: usize,
    pub(super) destination: usize,
    pub(super) length: usize,
}

pub(super) struct Layout {
    pub(super) commands: [CommandDescriptor; MAX_BASELINE_SERVER_COMMANDS],
    pub(super) command_count: usize,
    pub(super) record_count: u32,
    pub(super) byte_count: usize,
    pub(super) logical_bytes: usize,
}

pub(super) fn scan(
    body: &[u8],
    budget: ServiceControlBudget,
    scratch: &mut [u8],
    capacity: usize,
) -> Result<Layout, ServiceControlError> {
    if budget.max_records == 0 {
        return Err(ServiceControlError::InvalidBudget);
    }
    let mut reader = MessageReader::try_new(body).map_err(ServiceControlError::Message)?;
    let mut layout = Layout {
        commands: [CommandDescriptor::default(); MAX_BASELINE_SERVER_COMMANDS],
        command_count: 0,
        record_count: 0,
        byte_count: 0,
        logical_bytes: 0,
    };
    loop {
        if layout.record_count == budget.max_records {
            return Err(ServiceControlError::RecordBudget);
        }
        if reader.remaining_bytes() == 0 {
            return Err(ServiceControlError::MissingEof);
        }
        let opcode = reader.read_u8().map_err(ServiceControlError::Message)?;
        layout.record_count += 1;
        match opcode {
            0 => {}
            4 => {
                if layout.command_count == MAX_BASELINE_SERVER_COMMANDS {
                    return Err(ServiceControlError::CommandBudget);
                }
                let sequence = reader.read_i32().map_err(ServiceControlError::Message)?;
                if sequence <= 0 {
                    return Err(ServiceControlError::Sequence);
                }
                let source = body.len() - reader.remaining_bytes();
                let length = reader
                    .read_legacy_string(scratch, MessageStringLimit::Big)
                    .map_err(ServiceControlError::Message)?;
                let end = layout
                    .byte_count
                    .checked_add(length)
                    .filter(|&end| end <= capacity)
                    .ok_or(ServiceControlError::OutputCapacity)?;
                layout.commands[layout.command_count] = CommandDescriptor {
                    sequence: u32::try_from(sequence).map_err(|_| ServiceControlError::Sequence)?,
                    source,
                    destination: layout.byte_count,
                    length,
                };
                layout.command_count += 1;
                layout.byte_count = end;
            }
            7 => {
                layout.logical_bytes = body.len() - reader.remaining_bytes();
                return Ok(layout);
            }
            other => return Err(ServiceControlError::UnsupportedService(other)),
        }
    }
}
