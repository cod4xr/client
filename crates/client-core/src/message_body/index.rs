// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BodyReadError, BodyReader};
use cod4x_protocol::LegacyIndexDecoder;

impl BodyReader<'_> {
    /// Delegates a complete ordered legacy index field without widening its namespace.
    /// Decoder and both reader cursors commit together after successful view validation.
    ///
    /// # Errors
    /// Rejects shared index/primitive validation without changing either owner.
    pub fn read_index(&mut self, decoder: &mut LegacyIndexDecoder) -> Result<u16, BodyReadError> {
        let mut candidate = *decoder;
        let result =
            self.bit_field(|reader| candidate.read(reader).map_err(BodyReadError::Index))?;
        *decoder = candidate;
        Ok(result)
    }
}
