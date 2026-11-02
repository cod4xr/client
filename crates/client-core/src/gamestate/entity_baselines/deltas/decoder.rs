// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::LegacyMapCenter;
use super::{EntityDeltaError, PreparedEntityDelta};
use crate::{BaselineProtocol, message_body::BodyReader};
use cod4x_protocol::{LegacyIndexDecoder, LegacyIndexKind};
use std::fmt;

/// One ordered entity-index stream, independent of any fixed host entity array.
pub struct EntityDeltaDecoder {
    pub(super) indices: LegacyIndexDecoder,
    pub(super) map_center: Option<LegacyMapCenter>,
}
impl EntityDeltaDecoder {
    /// Starts a protocol-21 record stream. Discovery alone does not select a dialect.
    ///
    /// # Errors
    /// Rejects protocol 6 without explicit context; use [`Self::with_legacy_map_center`].
    pub fn new(profile: BaselineProtocol) -> Result<Self, EntityDeltaError> {
        if profile != BaselineProtocol::Cod4x21 {
            return Err(EntityDeltaError::UnsupportedProfile);
        }
        Ok(Self {
            indices: LegacyIndexDecoder::new(LegacyIndexKind::Entity),
            map_center: None,
        })
    }
    /// Starts an explicitly selected protocol-6 record stream with checked map context.
    #[must_use]
    pub fn with_legacy_map_center(center: LegacyMapCenter) -> Self {
        Self {
            indices: LegacyIndexDecoder::new(LegacyIndexKind::Entity),
            map_center: Some(center),
        }
    }
    /// Returns the original explicit coordinate profile, independent of discovery.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        if self.map_center.is_some() {
            BaselineProtocol::Legacy6
        } else {
            BaselineProtocol::Cod4x21
        }
    }
    /// Replaces legacy context between records without resetting index order.
    ///
    /// # Errors
    /// Rejects protocol-21 owners without changing profile, context or indices.
    pub fn set_legacy_map_center(
        &mut self,
        center: LegacyMapCenter,
    ) -> Result<(), EntityDeltaError> {
        *self
            .map_center
            .as_mut()
            .ok_or(EntityDeltaError::UnsupportedProfile)? = center;
        Ok(())
    }
    /// Restarts ordering for a new entity stream, retaining explicit coordinate context.
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
        f.debug_struct("EntityDeltaDecoder")
            .field("profile", &self.profile())
            .finish_non_exhaustive()
    }
}
