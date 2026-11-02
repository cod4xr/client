// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BodyReadError, BodyReader};
use cod4x_protocol::{MAX_MESSAGE_BYTES, MessageError, MessageStringLimit};

impl BodyReader<'_> {
    /// Reads one unsigned byte through the shared primitive codec.
    ///
    /// # Errors
    /// Rejects truncation without changing the reader.
    pub fn read_u8(&mut self) -> Result<u8, BodyReadError> {
        self.byte_field(|reader| reader.read_u8().map_err(BodyReadError::Message))
    }
    /// Reads one signed little-endian 16-bit field through the shared codec.
    ///
    /// # Errors
    /// Rejects truncation without changing the reader.
    pub fn read_i16(&mut self) -> Result<i16, BodyReadError> {
        self.byte_field(|reader| reader.read_i16().map_err(BodyReadError::Message))
    }
    /// Reads one signed little-endian 32-bit field through the shared codec.
    ///
    /// # Errors
    /// Rejects truncation without changing the reader.
    pub fn read_i32(&mut self) -> Result<i32, BodyReadError> {
        self.byte_field(|reader| reader.read_i32().map_err(BodyReadError::Message))
    }
    /// Reads one signed little-endian 64-bit field through the shared codec.
    ///
    /// # Errors
    /// Rejects truncation without changing the reader.
    pub fn read_i64(&mut self) -> Result<i64, BodyReadError> {
        self.byte_field(|reader| reader.read_i64().map_err(BodyReadError::Message))
    }
    /// Reads binary32 without changing its original bit pattern.
    ///
    /// # Errors
    /// Rejects truncation without changing the reader.
    pub fn read_f32(&mut self) -> Result<f32, BodyReadError> {
        self.byte_field(|reader| reader.read_f32().map_err(BodyReadError::Message))
    }
    /// Reads up to 32 legacy bits through the shared codec, preserving both cursors.
    ///
    /// # Errors
    /// Rejects invalid width or truncation without changing byte/cache state.
    pub fn read_bits(&mut self, width: u8) -> Result<u32, BodyReadError> {
        self.bit_field(|reader| reader.read_bits(width).map_err(BodyReadError::Message))
    }
    /// Copies an entire byte span through bounded shared views after full source preflight.
    /// Cached bits remain available. Output and reader stay unchanged on input failure.
    ///
    /// # Errors
    /// Rejects insufficient complete input before any output or cursor changes.
    pub fn read_bytes(&mut self, output: &mut [u8]) -> Result<(), BodyReadError> {
        if output.len() > self.remaining_bytes() {
            return Err(MessageError::Truncated {
                needed: output.len(),
                remaining: self.remaining_bytes(),
            }
            .into());
        }
        let mut candidate = self.clone();
        for chunk in output.chunks_mut(MAX_MESSAGE_BYTES) {
            candidate
                .byte_field(|reader| reader.read_bytes(chunk).map_err(BodyReadError::Message))?;
        }
        *self = candidate;
        Ok(())
    }
    /// Delegates native NUL framing, byte preservation and percent normalization.
    /// Shared string budgets remain field limits, independent of complete-body storage.
    ///
    /// # Errors
    /// Rejects missing terminators, profile overflow or output capacity atomically.
    pub fn read_legacy_string(
        &mut self,
        output: &mut [u8],
        limit: MessageStringLimit,
    ) -> Result<usize, BodyReadError> {
        self.byte_field(|reader| {
            reader
                .read_legacy_string(output, limit)
                .map_err(BodyReadError::Message)
        })
    }
}
