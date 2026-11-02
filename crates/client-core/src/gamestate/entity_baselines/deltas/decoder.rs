// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{EntityDeltaError, PreparedEntityDelta};
use crate::{BaselineProtocol, message_body::BodyReader};
use cod4x_protocol::{LegacyIndexDecoder, LegacyIndexKind};
use std::fmt;

/// One ordered entity-index stream, independent of any fixed host entity array.
pub struct EntityDeltaDecoder {
    pub(super) indices: LegacyIndexDecoder,
}
impl EntityDeltaDecoder {
    /// Starts a protocol-21 record stream. Discovery alone does not select a dialect.
    ///
    /// # Errors
    /// Rejects unreviewed coordinate profiles before creating an index owner.
    pub fn new(profile: BaselineProtocol) -> Result<Self, EntityDeltaError> {
        if profile != BaselineProtocol::Cod4x21 {
            return Err(EntityDeltaError::UnsupportedProfile);
        }
        Ok(Self {
            indices: LegacyIndexDecoder::new(LegacyIndexKind::Entity),
        })
    }
    /// Restarts ordering when the host explicitly begins a new entity stream.
    pub fn reset(&mut self) {
        self.indices.reset();
    }
    /// Prepares the next ordered index without advancing the reader or this decoder.
    /// The resulting number lets the host select a prior state without peer callbacks.
    /// Dropping preparation cancels; snapshot framing/terminators remain separate.
    ///
    /// # Errors
    /// Rejects truncated, unordered or out-of-namespace indices without changing either
    /// original owner. Later field/reference failure also preserves both originals.
    pub fn prepare<'owner, 'reader, 'body>(
        &'owner mut self,
        reader: &'reader mut BodyReader<'body>,
    ) -> Result<PreparedEntityDelta<'owner, 'reader, 'body>, EntityDeltaError> {
        let mut candidate = reader.clone();
        let mut indices = self.indices;
        let number = u32::from(candidate.read_index(&mut indices)?);
        Ok(PreparedEntityDelta {
            owner: self,
            reader,
            candidate,
            indices,
            number,
        })
    }
}
impl fmt::Debug for EntityDeltaDecoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntityDeltaDecoder").finish_non_exhaustive()
    }
}
