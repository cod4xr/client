// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Zone5Error;

pub(super) fn word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}
pub(super) struct Reader<'input> {
    input: &'input [u8],
    maximum: usize,
    position: usize,
}
impl<'input> Reader<'input> {
    pub fn new(input: &'input [u8], maximum: usize) -> Self {
        Self {
            input,
            maximum,
            position: 0,
        }
    }
    pub fn ensure(&self, count: usize) -> Result<(), Zone5Error> {
        let end = self
            .position
            .checked_add(count)
            .filter(|&n| n <= self.maximum)
            .ok_or(Zone5Error::PrefixBudget)?;
        self.input
            .get(self.position..end)
            .ok_or(Zone5Error::Truncated)?;
        Ok(())
    }
    pub fn take(&mut self, count: usize) -> Result<&'input [u8], Zone5Error> {
        let end = self
            .position
            .checked_add(count)
            .filter(|&n| n <= self.maximum)
            .ok_or(Zone5Error::PrefixBudget)?;
        let bytes = self
            .input
            .get(self.position..end)
            .ok_or(Zone5Error::Truncated)?;
        self.position = end;
        Ok(bytes)
    }
    pub fn string(&mut self, remaining: usize) -> Result<usize, Zone5Error> {
        let limit = self.input.len().min(self.maximum);
        let available = limit.saturating_sub(self.position).min(remaining);
        let bytes = self
            .input
            .get(self.position..self.position + available)
            .ok_or(Zone5Error::Truncated)?;
        let length = bytes
            .iter()
            .position(|&value| value == 0)
            .map(|n| n + 1)
            .ok_or({
                if available == remaining {
                    Zone5Error::StringByteBudget
                } else if limit == self.maximum {
                    Zone5Error::PrefixBudget
                } else {
                    Zone5Error::Truncated
                }
            })?;
        self.take(length)?;
        Ok(length)
    }
    pub fn position(&self) -> usize {
        self.position
    }
}
