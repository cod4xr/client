// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    field_error::FieldReadError,
    field_values::{raw, truncated_i32},
};
use crate::message_body::BodyReader;

pub(super) fn color(reader: &mut BodyReader<'_>, reference: u32) -> Result<u32, FieldReadError> {
    let mut bytes = reference.to_le_bytes();
    if reader.read_bits(1)? != 0 {
        // The native shortcut switches any nonzero alpha to zero, not XOR 255.
        bytes[3] = if bytes[3] == 0 { 255 } else { 0 };
    } else {
        if reader.read_bits(1)? == 0 {
            for byte in &mut bytes[..3] {
                *byte = reader.read_u8()?;
            }
        }
        bytes[3] = u8::try_from(reader.read_bits(5)? << 3).map_err(|_| FieldReadError::Encoding)?;
    }
    Ok(u32::from_le_bytes(bytes))
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "The native scale rule rounds the completed binary64 expression once to binary32."
)]
pub(super) fn scale(reader: &mut BodyReader<'_>) -> Result<u32, FieldReadError> {
    let index = reader.read_bits(5)?;
    // The reviewed qword base is an exact promoted binary32 1.4, not binary64 1.4.
    Ok(((f64::from(index) / 10.0 + f64::from(1.4_f32)) as f32).to_bits())
}

#[allow(
    clippy::cast_precision_loss,
    reason = "Native compact coordinates round the signed-32 result to binary32."
)]
pub(super) fn compact_float(
    reader: &mut BodyReader<'_>,
    reference: u32,
) -> Result<u32, FieldReadError> {
    if reader.read_bits(1)? == 0 {
        return Ok(0);
    }
    if reader.read_bits(1)? != 0 {
        return Ok(raw(reader)? ^ reference);
    }
    let packed = reader.read_bits(4)? | (u32::from(reader.read_u8()?) << 4);
    let prior = truncated_i32(f32::from_bits(reference)).ok_or(FieldReadError::ReferenceFloat)?;
    // Preserve the complete prior integer: unlike the ordinary 13-bit float rule,
    // the native 12-bit window is not restricted to a compact-range prior value.
    let biased = prior.wrapping_add(2048).cast_unsigned() ^ packed;
    Ok((biased.cast_signed().wrapping_sub(2048) as f32).to_bits())
}
