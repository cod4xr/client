// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{EntityDelta, EntityDeltaDecoder, EntityDeltaError, EntityState, record};
use crate::message_body::BodyReader;
use cod4x_protocol::LegacyIndexDecoder;
use std::fmt;

/// Exclusive index/reader lease for one record; dropping leaves both originals intact.
/// Neither owner can change during reference lookup:
/// ```compile_fail
/// use cod4xr_client_core::{entity_deltas::EntityDeltaDecoder, message_body::BodyReader};
/// fn cannot_reset(decoder: &mut EntityDeltaDecoder, reader: &mut BodyReader<'_>) {
///     if let Ok(prepared) = decoder.prepare(reader) {
///         decoder.reset();
///         let _ = prepared.decode(None, 0);
///     }
/// }
/// ```
/// ```compile_fail
/// use cod4xr_client_core::{entity_deltas::EntityDeltaDecoder, message_body::BodyReader};
/// fn cannot_advance(decoder: &mut EntityDeltaDecoder, reader: &mut BodyReader<'_>) {
///     if let Ok(prepared) = decoder.prepare(reader) {
///         let _ = reader.read_u8();
///         let _ = prepared.decode(None, 0);
///     }
/// }
/// ```
#[must_use = "decode with the selected reference, or drop to cancel"]
pub struct PreparedEntityDelta<'owner, 'reader, 'body> {
    pub(super) owner: &'owner mut EntityDeltaDecoder,
    pub(super) reader: &'reader mut BodyReader<'body>,
    pub(super) candidate: BodyReader<'body>,
    pub(super) indices: LegacyIndexDecoder,
    pub(super) number: u32,
}
impl PreparedEntityDelta<'_, '_, '_> {
    /// Returns the prepared number for caller-owned baseline/snapshot lookup.
    #[must_use]
    pub fn number(&self) -> u32 {
        self.number
    }
    /// Decodes the complete record against this entity's explicit prior state.
    /// `None` means a zero reference, never an implicit host baseline lookup.
    /// `time` supplies the full message-time bits for relative native time fields.
    /// Success commits both cursor/index owners and returns owned inert state/removal.
    /// It does not commit a complete snapshot, acknowledge a message or apply entities.
    ///
    /// # Errors
    /// Rejects reference-number mismatch, malformed counts/fields and truncation.
    /// Compact float forms reject prior values outside the writer's finite integer
    /// range; raw forms preserve arbitrary prior/target words, including NaNs.
    /// Changed legacy origins reject nonfinite prior values; full origin forms also
    /// require a truncated prior value representable in signed 32 bits. Short forms
    /// add to the finite prior float without imposing that integer-conversion bound.
    /// Every failure preserves both original owners, including cached unread bits.
    pub fn decode(
        mut self,
        reference: Option<&EntityState>,
        time: u32,
    ) -> Result<EntityDelta, EntityDeltaError> {
        if reference.is_some_and(|state| state.number() != self.number) {
            return Err(EntityDeltaError::ReferenceNumber);
        }
        let result = record::read(
            &mut self.candidate,
            self.number,
            reference,
            time,
            self.owner.map_center,
        )?;
        self.owner.indices = self.indices;
        *self.reader = self.candidate;
        Ok(result)
    }
}
impl fmt::Debug for PreparedEntityDelta<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedEntityDelta")
            .field("number", &self.number)
            .finish_non_exhaustive()
    }
}
