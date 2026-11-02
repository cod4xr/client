// SPDX-License-Identifier: AGPL-3.0-or-later
use super::field_error::FieldReadError;
use crate::message_body::BodyReader;

pub(super) fn raw(reader: &mut BodyReader<'_>) -> Result<u32, FieldReadError> {
    Ok(u32::from_le_bytes(reader.read_i32()?.to_le_bytes()))
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "Native truncated float references are checked against exact signed-32 bounds."
)]
pub(super) fn truncated_i32(value: f32) -> Option<i32> {
    let truncated = f64::from(value).trunc();
    if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&truncated) {
        return None;
    }
    Some(truncated as i32)
}
pub(super) fn integer(reader: &mut BodyReader<'_>, width: u8) -> Result<u32, FieldReadError> {
    // Low residual bits use the bit cursor, then whole bytes use the byte cursor.
    let residual = width % 8;
    let mut value = reader.read_bits(residual)?;
    for shift in (residual..width).step_by(8) {
        value |= u32::from(reader.read_u8()?) << shift;
    }
    Ok(value)
}
pub(super) fn signed_integer(
    reader: &mut BodyReader<'_>,
    width: u8,
    reference: u32,
) -> Result<u32, FieldReadError> {
    if reader.read_bits(1)? == 0 {
        return Ok(0);
    }
    let mask = u32::MAX >> (32 - u32::from(width));
    let value = integer(reader, width)? ^ (reference & mask);
    let shift = 32 - u32::from(width);
    Ok(((value << shift).cast_signed() >> shift).cast_unsigned())
}
pub(super) fn angle(reader: &mut BodyReader<'_>) -> Result<u32, FieldReadError> {
    Ok((f32::from(reader.read_i16()?) * (360.0 / 65536.0)).to_bits())
}
pub(super) fn float(reader: &mut BodyReader<'_>, reference: u32) -> Result<u32, FieldReadError> {
    if reader.read_bits(1)? == 0 {
        return Ok(reader.read_bits(1)? << 31);
    }
    if reader.read_bits(1)? == 1 {
        return Ok(raw(reader)? ^ reference);
    }
    let packed = reader.read_bits(5)? | (u32::from(reader.read_u8()?) << 5);
    let biased =
        i16::try_from(packed ^ float_bias(reference)?).map_err(|_| FieldReadError::Encoding)?;
    Ok(f32::from(biased - 4096).to_bits())
}
#[allow(
    clippy::cast_possible_truncation,
    reason = "Finite truncated reference is checked within the exact compact integer range."
)]
fn float_bias(reference: u32) -> Result<u32, FieldReadError> {
    let truncated = f32::from_bits(reference).trunc();
    if !truncated.is_finite() || !(-4096.0..4096.0).contains(&truncated) {
        return Err(FieldReadError::ReferenceFloat);
    }
    let integer = truncated as i16;
    u32::try_from(integer + 4096).map_err(|_| FieldReadError::Encoding)
}
pub(super) fn flags(reader: &mut BodyReader<'_>, reference: u32) -> Result<u32, FieldReadError> {
    if reader.read_bits(1)? == 0 {
        return Ok(reference ^ (1_u32 << reader.read_bits(5)?));
    }
    let mut value = 0;
    for shift in [0, 8, 16] {
        value |= u32::from(reader.read_u8()?) << shift;
    }
    Ok(value)
}
pub(super) fn ground(reader: &mut BodyReader<'_>) -> Result<u32, FieldReadError> {
    if reader.read_bits(1)? == 1 {
        return Ok(1022);
    }
    if reader.read_bits(1)? == 1 {
        return Ok(0);
    }
    Ok(reader.read_bits(2)? | (u32::from(reader.read_u8()?) << 2))
}
