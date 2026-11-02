// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    MAX_BASELINE_SERVER_COMMANDS, ServiceControlBudget, ServiceControlError, client_scan::Clients,
};
use crate::message_body::BodyReader;
use cod4x_protocol::MessageStringLimit;

#[derive(Clone, Copy, Default)]
pub(crate) struct CommandDescriptor {
    pub sequence: u32,
    pub source: usize,
    pub destination: usize,
    pub length: usize,
}
pub(crate) struct Layout {
    pub commands: [CommandDescriptor; MAX_BASELINE_SERVER_COMMANDS],
    pub command_count: usize,
    pub record_count: u32,
    pub byte_count: usize,
    pub logical_bytes: usize,
    pub client_update_count: usize,
}
impl Layout {
    pub(crate) fn new(budget: ServiceControlBudget) -> Result<Self, ServiceControlError> {
        if budget.max_records == 0 {
            return Err(ServiceControlError::InvalidBudget);
        }
        Ok(Self {
            commands: [CommandDescriptor::default(); MAX_BASELINE_SERVER_COMMANDS],
            command_count: 0,
            record_count: 0,
            byte_count: 0,
            logical_bytes: 0,
            client_update_count: 0,
        })
    }
    pub(crate) fn next(
        &mut self,
        reader: &mut BodyReader<'_>,
        budget: ServiceControlBudget,
    ) -> Result<u8, ServiceControlError> {
        if self.record_count >= budget.max_records {
            return Err(ServiceControlError::RecordBudget);
        }
        if reader.remaining_bytes() == 0 {
            return Err(ServiceControlError::MissingEof);
        }
        let opcode = reader.read_u8().map_err(ServiceControlError::from)?;
        self.record_count += 1;
        Ok(opcode)
    }
    pub(crate) fn control(
        &mut self,
        opcode: u8,
        reader: &mut BodyReader<'_>,
        scratch: &mut [u8],
        capacity: usize,
        clients: &mut Clients<'_>,
    ) -> Result<bool, ServiceControlError> {
        match opcode {
            0 => {}
            4 => self.command(reader, scratch, capacity)?,
            7 => {
                self.logical_bytes = reader.byte_position();
                return Ok(true);
            }
            11 => {
                clients.read(self.client_update_count, reader)?;
                self.client_update_count += 1;
            }
            other => return Err(ServiceControlError::UnsupportedService(other)),
        }
        Ok(false)
    }
    fn command(
        &mut self,
        reader: &mut BodyReader<'_>,
        scratch: &mut [u8],
        capacity: usize,
    ) -> Result<(), ServiceControlError> {
        if self.command_count == MAX_BASELINE_SERVER_COMMANDS {
            return Err(ServiceControlError::CommandBudget);
        }
        let sequence = reader.read_i32().map_err(ServiceControlError::from)?;
        if sequence <= 0 {
            return Err(ServiceControlError::Sequence);
        }
        let source = reader.byte_position();
        let length = reader
            .read_legacy_string(scratch, MessageStringLimit::Big)
            .map_err(ServiceControlError::from)?;
        let end = self
            .byte_count
            .checked_add(length)
            .filter(|&end| end <= capacity)
            .ok_or(ServiceControlError::OutputCapacity)?;
        self.commands[self.command_count] = CommandDescriptor {
            sequence: sequence.cast_unsigned(),
            source,
            destination: self.byte_count,
            length,
        };
        self.command_count += 1;
        self.byte_count = end;
        Ok(())
    }
    pub(crate) fn newest_command(&self, initial: u32) -> u32 {
        self.commands[..self.command_count]
            .iter()
            .fold(initial, |newest, command| newest.max(command.sequence))
    }
}
pub(super) fn scan(
    body: &[u8],
    budget: ServiceControlBudget,
    scratch: &mut [u8],
    capacity: usize,
    clients: &mut Clients<'_>,
) -> Result<Layout, ServiceControlError> {
    let mut layout = Layout::new(budget)?;
    let mut reader = BodyReader::try_new(body, body.len()).map_err(ServiceControlError::from)?;
    loop {
        let opcode = layout.next(&mut reader, budget)?;
        if layout.control(opcode, &mut reader, scratch, capacity, clients)? {
            return Ok(layout);
        }
    }
}
