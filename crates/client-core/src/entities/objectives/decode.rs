// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::fields::read_value;
use super::{OBJECTIVE_SLOTS, ObjectiveError, ObjectiveState, table};
use crate::message_body::BodyReader;

pub(super) fn read(
    reader: &mut BodyReader<'_>,
    objectives: &mut [ObjectiveState; OBJECTIVE_SLOTS],
) -> Result<(), ObjectiveError> {
    if reader.read_bits(1)? == 0 {
        return Ok(());
    }
    for objective in objectives {
        objective.status =
            u8::try_from(reader.read_bits(3)?).map_err(|_| ObjectiveError::FieldEncoding)?;
        if reader.read_bits(1)? != 0 {
            // Every changed entry has all six per-field flags, not a prefix count.
            // Status was transmitted separately even when these words are unchanged.
            for field in &table::FIELDS {
                let index = field.word as usize;
                objective.words[index] =
                    read_value(reader, field.value, objective.words[index], 0, None)?;
            }
        }
    }
    Ok(())
}
