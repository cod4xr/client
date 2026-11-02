// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Pe32Error;

pub(super) fn span(input: &[u8], start: usize, length: usize) -> Result<&[u8], Pe32Error> {
    let end = start.checked_add(length).ok_or(Pe32Error::Truncated)?;
    input.get(start..end).ok_or(Pe32Error::Truncated)
}
pub(super) fn short(input: &[u8], offset: usize) -> Result<u16, Pe32Error> {
    let bytes = span(input, offset, 2)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}
pub(super) fn word(input: &[u8], offset: usize) -> Result<u32, Pe32Error> {
    let bytes = span(input, offset, 4)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}
pub(super) fn host(value: u32) -> Result<usize, Pe32Error> {
    usize::try_from(value).map_err(|_| Pe32Error::Truncated)
}
