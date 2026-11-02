// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BaselineWord, EntityBaselineError,
    field_error::FieldReadError,
    field_values::{flags, float, ground, integer, raw},
};
use crate::message_body::BodyReader;

#[derive(Clone, Copy)]
pub(super) struct FieldSpec {
    pub word: BaselineWord,
    bits: i16,
    unconditional: bool,
}

pub(super) const fn f(word: BaselineWord, bits: i16, unconditional: bool) -> FieldSpec {
    FieldSpec {
        word,
        bits,
        unconditional,
    }
}

pub(super) fn read(
    reader: &mut BodyReader<'_>,
    field: FieldSpec,
) -> Result<u32, EntityBaselineError> {
    read_delta(reader, field, 0, 0).map_err(EntityBaselineError::from)
}

pub(super) fn read_delta(
    reader: &mut BodyReader<'_>,
    field: FieldSpec,
    reference: u32,
    time: u32,
) -> Result<u32, FieldReadError> {
    if !field.unconditional && reader.read_bits(1)? == 0 {
        return Ok(reference);
    }
    Ok(match field.bits {
        0 => float(reader, reference)?,
        -100 => {
            if reader.read_bits(1)? == 0 {
                0
            } else {
                let angle = reader.read_i16()?;
                (f32::from(angle) * (360.0 / 65536.0)).to_bits()
            }
        }
        -98 => flags(reader, reference)?,
        -97 => {
            if reader.read_bits(1)? == 0 {
                // Relative time uses the message time, independent of reference state.
                time.wrapping_sub(reader.read_bits(8)?)
            } else {
                raw(reader)?
            }
        }
        -96 => ground(reader)?,
        -94 | -93 => u32::from(reader.read_u8()?),
        // Protocol 21 writes coordinates directly as binary32, without legacy map bias.
        -92..=-90 => raw(reader)?,
        -8 => {
            if reader.read_bits(1)? == 0 {
                0
            } else {
                let signed = i32::from(i8::from_le_bytes([
                    reader.read_u8()? ^ reference.to_le_bytes()[0]
                ]));
                u32::from_le_bytes(signed.to_le_bytes())
            }
        }
        1..=32 => {
            if reader.read_bits(1)? == 0 {
                0
            } else {
                let width = u8::try_from(field.bits).map_err(|_| FieldReadError::Encoding)?;
                let mask = u32::MAX >> (32 - u32::from(width));
                integer(reader, width)? ^ (reference & mask)
            }
        }
        _ => return Err(FieldReadError::Encoding),
    })
}
