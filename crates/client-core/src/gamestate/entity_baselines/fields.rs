// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BaselineWord, EntityBaselineError};
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
    if !field.unconditional && reader.read_bits(1)? == 0 {
        return Ok(0);
    }
    Ok(match field.bits {
        0 => float(reader)?,
        -100 => {
            if reader.read_bits(1)? == 0 {
                0
            } else {
                let angle = u16::from_le_bytes(reader.read_i16()?.to_le_bytes());
                (f32::from(angle) * (360.0 / 65536.0)).to_bits()
            }
        }
        -98 => flags(reader)?,
        -97 => {
            if reader.read_bits(1)? == 0 {
                // Gamestate baselines use time zero. Preserve native 32-bit time bits.
                0_u32.wrapping_sub(reader.read_bits(8)?)
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
                let signed = i32::from(i8::from_le_bytes([reader.read_u8()?]));
                u32::from_le_bytes(signed.to_le_bytes())
            }
        }
        1..=32 => {
            if reader.read_bits(1)? == 0 {
                0
            } else {
                let width =
                    u8::try_from(field.bits).map_err(|_| EntityBaselineError::FieldEncoding)?;
                integer(reader, width)?
            }
        }
        _ => return Err(EntityBaselineError::FieldEncoding),
    })
}

fn raw(reader: &mut BodyReader<'_>) -> Result<u32, EntityBaselineError> {
    Ok(u32::from_le_bytes(reader.read_i32()?.to_le_bytes()))
}
fn integer(reader: &mut BodyReader<'_>, width: u8) -> Result<u32, EntityBaselineError> {
    // Native integers put the low residual bits in the bit cursor, then whole bytes
    // at the independent byte cursor; read_bits(width) would be the wrong format.
    let residual = width % 8;
    let mut value = reader.read_bits(residual)?;
    for shift in (residual..width).step_by(8) {
        value |= u32::from(reader.read_u8()?) << shift;
    }
    Ok(value)
}
fn float(reader: &mut BodyReader<'_>) -> Result<u32, EntityBaselineError> {
    if reader.read_bits(1)? == 0 {
        return Ok(reader.read_bits(1)? << 31);
    }
    if reader.read_bits(1)? == 1 {
        return raw(reader);
    }
    let packed = reader.read_bits(5)? | (u32::from(reader.read_u8()?) << 5);
    let biased = i16::try_from(packed ^ 4096).map_err(|_| EntityBaselineError::FieldEncoding)?;
    Ok(f32::from(biased - 4096).to_bits())
}
fn flags(reader: &mut BodyReader<'_>) -> Result<u32, EntityBaselineError> {
    if reader.read_bits(1)? == 0 {
        return Ok(1_u32 << reader.read_bits(5)?);
    }
    let mut value = 0;
    for shift in [0, 8, 16] {
        value |= u32::from(reader.read_u8()?) << shift;
    }
    Ok(value)
}
fn ground(reader: &mut BodyReader<'_>) -> Result<u32, EntityBaselineError> {
    if reader.read_bits(1)? == 1 {
        return Ok(1022);
    }
    if reader.read_bits(1)? == 1 {
        return Ok(0);
    }
    Ok(reader.read_bits(2)? | (u32::from(reader.read_u8()?) << 2))
}
