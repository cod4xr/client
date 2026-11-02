// SPDX-License-Identifier: AGPL-3.0-or-later
use super::WaveletError;

/// Borrowed LSB-first cursor, with raw prefix bytes followed by coded bits.
pub(super) struct Bits<'input> {
    input: &'input [u8],
    byte: usize,
    bit: u8,
    coded: bool,
}
impl<'input> Bits<'input> {
    pub fn new(input: &'input [u8]) -> Self {
        Self {
            input,
            byte: 0,
            bit: 0,
            coded: false,
        }
    }
    pub fn raw(&mut self) -> Result<u8, WaveletError> {
        if self.coded {
            return Err(WaveletError::RawAfterBits);
        }
        let value = *self.input.get(self.byte).ok_or(WaveletError::Truncated)?;
        self.byte += 1;
        Ok(value)
    }
    /// Zero extends lookahead only; consuming a codeword still requires real bits.
    pub fn peek(&self, count: u8) -> u16 {
        let mut value = 0;
        let mut byte = self.byte;
        let mut bit = self.bit;
        for shift in 0..count {
            if let Some(&source) = self.input.get(byte) {
                value |= u16::from((source >> bit) & 1) << shift;
            }
            bit += 1;
            if bit == 8 {
                bit = 0;
                byte += 1;
            }
        }
        value
    }
    pub fn take(&mut self, count: u8) -> Result<u16, WaveletError> {
        if count > 12 {
            return Err(WaveletError::BitCount);
        }
        let bits = usize::from(self.bit) + usize::from(count);
        let end_byte = self
            .byte
            .checked_add(bits / 8)
            .ok_or(WaveletError::Truncated)?;
        if end_byte > self.input.len() || (end_byte == self.input.len() && bits % 8 != 0) {
            return Err(WaveletError::Truncated);
        }
        let result = self.peek(count);
        self.byte = end_byte;
        self.bit = u8::try_from(bits % 8).map_err(|_| WaveletError::BitCount)?;
        self.coded = true;
        Ok(result)
    }
    pub fn consumed(&self) -> (usize, u8) {
        (self.byte, self.bit)
    }
}
