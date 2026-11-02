// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{LegacyMapCenter, field_error::FieldReadError};
use crate::message_body::BodyReader;

#[allow(
    clippy::cast_precision_loss,
    reason = "Native full-coordinate decoding converts its signed-32 arithmetic result to binary32."
)]
pub(super) fn read(
    reader: &mut BodyReader<'_>,
    encoding: i16,
    center: LegacyMapCenter,
    reference: u32,
) -> Result<u32, FieldReadError> {
    let axis = match encoding {
        -92 => 0,
        -91 => 1,
        -90 => 2,
        _ => return Err(FieldReadError::Encoding),
    };
    let prior_float = f32::from_bits(reference);
    if !prior_float.is_finite() {
        return Err(FieldReadError::ReferenceCoordinate);
    }
    if reader.read_bits(1)? == 0 {
        let encoded = i16::try_from(reader.read_bits(7)?).map_err(|_| FieldReadError::Encoding)?;
        return Ok((prior_float + f32::from(encoded - 64)).to_bits());
    }
    let prior_integer = integer_reference(prior_float)?;
    let encoded = reader.read_bits(16)?;
    let bias = center.biases[axis];
    // Preserve full signed bias bits: masking this prior value to 16 bits would
    // change the reviewed reader's coordinate window. Wrapping matches its x86
    // arithmetic without relying on C signed-overflow behavior.
    let prior = prior_integer.wrapping_sub(bias).wrapping_add(32_768);
    let coordinate = (prior.cast_unsigned() ^ encoded)
        .cast_signed()
        .wrapping_add(bias)
        .wrapping_sub(32_768);
    Ok((coordinate as f32).to_bits())
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "Finite truncated prior coordinate is checked against exact signed-32 bounds."
)]
fn integer_reference(value: f32) -> Result<i32, FieldReadError> {
    let truncated = f64::from(value).trunc();
    if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&truncated) {
        return Err(FieldReadError::ReferenceCoordinate);
    }
    Ok(truncated as i32)
}
