// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::fields::read_value;
use super::{PlayerScalarError, PlayerScalars, table};
use crate::message_body::BodyReader;

pub(super) fn read(
    reader: &mut BodyReader<'_>,
    time: u32,
    origins_sent: bool,
    count: usize,
    scalars: &mut PlayerScalars,
) -> Result<(), PlayerScalarError> {
    let fields = table::FIELDS
        .get(..count)
        .ok_or(PlayerScalarError::FieldCount)?;
    for field in fields {
        let index = field.word.index();
        let reference = if origins_sent && field.zero_when_origins_sent {
            0
        } else {
            scalars.words[index]
        };
        // Live hint-3 transmission zeros the reference, but still reads the outer
        // changed bit. Only hint 2 removes that bit in the native field reader.
        scalars.words[index] = read_value(reader, field.value, reference, time, None)?;
    }
    Ok(())
}
