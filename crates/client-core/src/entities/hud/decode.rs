// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{EntityDecodeContext, fields::read_value};
use super::{HUD_SLOTS, HudElement, HudError, HudWord, table};
use crate::message_body::BodyReader;

pub(super) fn array(
    reader: &mut BodyReader<'_>,
    context: EntityDecodeContext,
    time: u32,
    elements: &mut [HudElement; HUD_SLOTS],
) -> Result<(), HudError> {
    let count = usize::try_from(reader.read_bits(5)?).map_err(|_| HudError::FieldCount)?;
    for element in &mut elements[..count] {
        let last = usize::try_from(reader.read_bits(6)?).map_err(|_| HudError::FieldCount)?;
        let fields = table::FIELDS.get(..=last).ok_or(HudError::FieldCount)?;
        for field in fields {
            let index = field.word as usize;
            element.words[index] = read_value(
                reader,
                field.value,
                element.words[index],
                time,
                context.legacy_center(),
            )?;
        }
    }
    // Match native clearing exactly: the first already-free slot stops clearing.
    // Its inert words and every later slot survive for subsequent delta references.
    for element in &mut elements[count..] {
        if element.word(HudWord::Kind) == 0 {
            break;
        }
        *element = HudElement::default();
    }
    Ok(())
}
