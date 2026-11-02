// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BaselineWord, EntityBaselineError, LegacyMapCenter,
    field_error::FieldReadError,
    field_values::{angle, flags, float, ground, integer, raw, signed_integer},
    hud_values, legacy_coordinates,
};
use crate::message_body::BodyReader;

#[derive(Clone, Copy)]
pub(super) struct FieldSpec {
    pub word: BaselineWord,
    value: ValueSpec,
}

#[derive(Clone, Copy)]
pub(super) struct ValueSpec {
    bits: i16,
    unconditional: bool,
}

pub(super) const fn f(word: BaselineWord, bits: i16, unconditional: bool) -> FieldSpec {
    FieldSpec {
        word,
        value: v(bits, unconditional),
    }
}

pub(super) const fn v(bits: i16, unconditional: bool) -> ValueSpec {
    ValueSpec {
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
    read_delta_in_context(reader, field, reference, time, None)
}

pub(super) fn read_legacy(
    reader: &mut BodyReader<'_>,
    field: FieldSpec,
    center: LegacyMapCenter,
) -> Result<u32, EntityBaselineError> {
    read_delta_in_context(reader, field, 0, 0, Some(center)).map_err(EntityBaselineError::from)
}

pub(super) fn read_delta_in_context(
    reader: &mut BodyReader<'_>,
    field: FieldSpec,
    reference: u32,
    time: u32,
    legacy_center: Option<LegacyMapCenter>,
) -> Result<u32, FieldReadError> {
    read_value(reader, field.value, reference, time, legacy_center)
}

pub(super) fn read_value(
    reader: &mut BodyReader<'_>,
    field: ValueSpec,
    reference: u32,
    time: u32,
    legacy_center: Option<LegacyMapCenter>,
) -> Result<u32, FieldReadError> {
    if !field.unconditional && reader.read_bits(1)? == 0 {
        return Ok(reference);
    }
    Ok(match field.bits {
        0 => float(reader, reference)?,
        -99 => hud_values::compact_float(reader, reference)?,
        -86 => hud_values::scale(reader)?,
        -85 => hud_values::color(reader, reference)?,
        -88 => raw(reader)? ^ reference,
        -87 => angle(reader)?,
        -100 => {
            if reader.read_bits(1)? == 0 {
                0
            } else {
                angle(reader)?
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
        -92..=-90 => match legacy_center {
            Some(center) => legacy_coordinates::read(reader, field.bits, center, reference)?,
            // Protocol 21 writes absolute binary32 without legacy map bias.
            None => raw(reader)?,
        },
        -16 | -8 => signed_integer(
            reader,
            u8::try_from(-field.bits).map_err(|_| FieldReadError::Encoding)?,
            reference,
        )?,
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
