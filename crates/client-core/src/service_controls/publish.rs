// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{ServiceControlError, scan::Layout};
use cod4x_protocol::{MessageReader, MessageStringLimit};

pub(crate) fn commands(
    body: &[u8],
    layout: &Layout,
    output: &mut [u8],
) -> Result<(), ServiceControlError> {
    for descriptor in &layout.commands[..layout.command_count] {
        // Immutable source and every destination were proven by the complete shared scan.
        let source = &body[descriptor.source..=descriptor.source + descriptor.length];
        let target =
            &mut output[descriptor.destination..descriptor.destination + descriptor.length];
        let mut reader = MessageReader::try_new(source).map_err(ServiceControlError::Message)?;
        reader
            .read_legacy_string(target, MessageStringLimit::Big)
            .map_err(ServiceControlError::Message)?;
    }
    Ok(())
}
