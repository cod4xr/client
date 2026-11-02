// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Iwi6Error;

pub(super) fn span(input: &[u8], offset: usize, length: usize) -> Result<&[u8], Iwi6Error> {
    let end = offset.checked_add(length).ok_or(Iwi6Error::Extent)?;
    input.get(offset..end).ok_or(Iwi6Error::Truncated)
}
pub(super) fn short(input: &[u8], offset: usize) -> Result<u16, Iwi6Error> {
    let bytes = span(input, offset, 2)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}
pub(super) fn word(input: &[u8], offset: usize) -> Result<u32, Iwi6Error> {
    let bytes = span(input, offset, 4)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}
