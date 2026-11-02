// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{BASELINE_ENTITY_WORDS, EntityBaseline, LegacyMapCenter, fields, tables};
use super::{EntityDelta, EntityDeltaError, EntityState};
use crate::message_body::BodyReader;

pub(super) fn read(
    reader: &mut BodyReader<'_>,
    number: u32,
    reference: Option<&EntityState>,
    time: u32,
    map_center: Option<LegacyMapCenter>,
) -> Result<EntityDelta, EntityDeltaError> {
    if reader.read_bits(1)? != 0 {
        return Ok(EntityDelta::Removed { number });
    }
    let mut state = reference.map_or_else(
        || EntityBaseline {
            number,
            words: [0; BASELINE_ENTITY_WORDS],
        },
        |reference| reference.0.clone(),
    );
    if reader.read_bits(1)? != 0 {
        let count =
            usize::try_from(reader.read_bits(6)?).map_err(|_| EntityDeltaError::FieldCount)?;
        if count == 0 {
            return Err(EntityDeltaError::FieldCount);
        }
        let first = tables::select(0)[0];
        let kind = fields::read_delta_in_context(
            reader,
            first,
            state.words[first.word as usize],
            time,
            map_center,
        )?;
        state.words[first.word as usize] = kind;
        let table = tables::select(kind);
        if count > table.len() {
            return Err(EntityDeltaError::FieldCount);
        }
        for &field in &table[1..count] {
            let index = field.word as usize;
            state.words[index] =
                fields::read_delta_in_context(reader, field, state.words[index], time, map_center)?;
        }
    }
    Ok(EntityDelta::State(EntityState(state)))
}
