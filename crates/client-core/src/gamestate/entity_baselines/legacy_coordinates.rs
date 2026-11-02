// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{LegacyMapCenter, field_error::FieldReadError};
use crate::message_body::BodyReader;

#[allow(
    clippy::cast_precision_loss,
    reason = "Zero-reference XOR changes only low 16 bits; the signed difference fits 17 bits and binary32 exactly."
)]
pub(super) fn read_zero(
    reader: &mut BodyReader<'_>,
    encoding: i16,
    center: LegacyMapCenter,
) -> Result<u32, FieldReadError> {
    let axis = match encoding {
        -92 => 0,
        -91 => 1,
        -90 => 2,
        _ => return Err(FieldReadError::Encoding),
    };
    if reader.read_bits(1)? == 0 {
        let encoded = i16::try_from(reader.read_bits(7)?).map_err(|_| FieldReadError::Encoding)?;
        return Ok(f32::from(encoded - 64).to_bits());
    }
    let encoded = reader.read_bits(16)?;
    let bias = center.biases[axis];
    // Preserve full signed bias bits: masking this prior value to 16 bits would
    // change the reviewed reader's coordinate window. Wrapping matches its x86
    // arithmetic without relying on C signed-overflow behavior.
    let prior = 0_i32.wrapping_sub(bias).wrapping_add(32_768);
    let coordinate = (prior.cast_unsigned() ^ encoded)
        .cast_signed()
        .wrapping_add(bias)
        .wrapping_sub(32_768);
    Ok((coordinate as f32).to_bits())
}
