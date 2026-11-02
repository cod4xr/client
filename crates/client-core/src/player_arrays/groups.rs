// SPDX-License-Identifier: AGPL-3.0-or-later
use super::PlayerArrayError;
use crate::message_body::BodyReader;

// Closed callers supply only full static sixteen-entry groups. Every set bit
// carries an absolute signed short, never an XOR residual or a sparse index.
pub(super) fn read<const SLOTS: usize>(
    reader: &mut BodyReader<'_>,
    values: &mut [i32; SLOTS],
) -> Result<(), PlayerArrayError> {
    const {
        assert!(SLOTS.is_multiple_of(16));
    }
    for group in values.chunks_exact_mut(16) {
        if reader.read_bits(1)? == 0 {
            continue;
        }
        let mask = reader.read_i16()?.cast_unsigned();
        for (slot, value) in group.iter_mut().enumerate() {
            if mask & (1 << slot) != 0 {
                *value = i32::from(reader.read_i16()?);
            }
        }
    }
    Ok(())
}
