// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BodyReadError, BodyReader};
use cod4x_protocol::{MAX_MESSAGE_BYTES, MessageError, MessageReader};

impl BodyReader<'_> {
    // Only byte-cursor operations may use this fast path. The adapter's cached
    // bit byte remains untouched, matching shared/native interleaved semantics.
    pub(super) fn byte_field<T>(
        &mut self,
        read: impl FnOnce(&mut MessageReader<'_>) -> Result<T, BodyReadError>,
    ) -> Result<T, BodyReadError> {
        let length = self.remaining_bytes().min(MAX_MESSAGE_BYTES);
        let mut reader =
            MessageReader::try_new(&self.input[self.position..self.position + length])?;
        let result = read(&mut reader)?;
        let consumed = length
            .checked_sub(reader.remaining_bytes())
            .ok_or(BodyReadError::Cursor)?;
        self.position += consumed;
        Ok(result)
    }

    // Mixed/bit fields see a synthetic cached-byte slice followed by the actual
    // fresh bytes at the independent byte cursor. All extraction/ordering stays
    // inside the pinned shared reader, including concrete legacy index decoders.
    pub(super) fn bit_field<T>(
        &mut self,
        read: impl FnOnce(&mut MessageReader<'_>) -> Result<T, BodyReadError>,
    ) -> Result<T, BodyReadError> {
        let cached = [self.cached_byte];
        let prefix = if self.cached_bits == 0 {
            &[][..]
        } else {
            &cached[..]
        };
        let length = self.remaining_bytes().min(MAX_MESSAGE_BYTES - prefix.len());
        let mut reader = MessageReader::try_new_split(
            prefix,
            &self.input[self.position..self.position + length],
        )?;
        if self.cached_bits != 0 {
            let _ = reader.read_bits(8 - self.cached_bits)?;
        }
        let before = reader.remaining_bytes();
        let result = read(&mut reader)?;
        let consumed = before
            .checked_sub(reader.remaining_bytes())
            .ok_or(BodyReadError::Cursor)?;
        let (cached_byte, cached_bits) = cached_tail(&reader)?;
        self.position += consumed;
        self.cached_byte = cached_byte;
        self.cached_bits = cached_bits;
        Ok(result)
    }
}

// The shared API intentionally hides its cursor representation. Up to three
// cloned non-consuming probes recover cache length without duplicating bit
// extraction. A final delegated read captures only those existing cached bits.
fn cached_tail(reader: &MessageReader<'_>) -> Result<(u8, u8), BodyReadError> {
    let mut lower = 0_u8;
    let mut upper = 8_u8;
    while lower + 1 < upper {
        let width = lower + (upper - lower) / 2;
        if has_cached(reader, width)? {
            lower = width;
        } else {
            upper = width;
        }
    }
    if lower == 0 {
        return Ok((0, 0));
    }
    let value = reader.clone().read_bits(lower)?;
    let value = u8::try_from(value).map_err(|_| BodyReadError::Cursor)?;
    Ok((value << (8 - lower), lower))
}

fn has_cached(reader: &MessageReader<'_>, width: u8) -> Result<bool, BodyReadError> {
    let mut probe = reader.clone();
    let before = probe.remaining_bytes();
    match probe.read_bits(width) {
        Ok(_) => Ok(probe.remaining_bytes() == before),
        Err(MessageError::Truncated { .. }) => Ok(false),
        Err(error) => Err(error.into()),
    }
}
