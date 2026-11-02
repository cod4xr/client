// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::fields::read_value;
use super::{CLIENT_STATE_WORDS, ClientState, ClientStateDelta, ClientStateError, table};
use crate::message_body::BodyReader;

pub(super) fn read(
    reader: &mut BodyReader<'_>,
    number: u32,
    reference: Option<&ClientState>,
) -> Result<ClientStateDelta, ClientStateError> {
    if reader.read_bits(1)? != 0 {
        return Ok(ClientStateDelta::Removed { number });
    }
    let mut state = reference.cloned().unwrap_or(ClientState {
        number,
        words: [0; CLIENT_STATE_WORDS],
    });
    if reader.read_bits(1)? != 0 {
        let count =
            usize::try_from(reader.read_bits(5)?).map_err(|_| ClientStateError::FieldCount)?;
        let fields = table::FIELDS
            .get(..count)
            .ok_or(ClientStateError::FieldCount)?;
        // The native reader accepts count zero as an empty prefix, despite the
        // ordinary writer preferring its explicit no-delta form for this case.
        for field in fields {
            let index = field.word as usize;
            state.words[index] = read_value(reader, field.value, state.words[index], 0, None)?;
        }
    }
    Ok(ClientStateDelta::State(state))
}
