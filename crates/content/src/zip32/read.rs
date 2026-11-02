// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Zip32Error;

pub(super) fn add(start: usize, length: usize) -> Result<usize, Zip32Error> {
    start.checked_add(length).ok_or(Zip32Error::Bounds)
}
pub(super) fn span(input: &[u8], start: usize, length: usize) -> Result<&[u8], Zip32Error> {
    input
        .get(start..add(start, length)?)
        .ok_or(Zip32Error::Bounds)
}
pub(super) fn short(input: &[u8], offset: usize) -> Result<u16, Zip32Error> {
    let bytes = span(input, offset, 2)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}
pub(super) fn word(input: &[u8], offset: usize) -> Result<u32, Zip32Error> {
    let bytes = span(input, offset, 4)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}
pub(super) fn host(value: u32) -> Result<usize, Zip32Error> {
    usize::try_from(value).map_err(|_| Zip32Error::Bounds)
}
