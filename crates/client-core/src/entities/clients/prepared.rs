// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{ClientState, ClientStateDecoder, ClientStateDelta, ClientStateError, record};
use crate::message_body::BodyReader;
use cod4x_protocol::LegacyIndexDecoder;
use std::fmt;

/// Exclusive record/reference lease; dropping cancels the more bit, index and fields.
/// The decoder cannot be reset during reference selection:
/// ```compile_fail
/// use cod4xr_client_core::{client_states::ClientStateDecoder, message_body::BodyReader};
/// fn cannot_reset(owner: &mut ClientStateDecoder, reader: &mut BodyReader<'_>) {
///     if let Ok(Some(prepared)) = owner.prepare(reader) {
///         owner.reset();
///         let _ = prepared.decode(None);
///     }
/// }
/// ```
/// Nor can the body advance:
/// ```compile_fail
/// use cod4xr_client_core::{client_states::ClientStateDecoder, message_body::BodyReader};
/// fn cannot_advance(owner: &mut ClientStateDecoder, reader: &mut BodyReader<'_>) {
///     if let Ok(Some(prepared)) = owner.prepare(reader) {
///         let _ = reader.read_u8();
///         let _ = prepared.decode(None);
///     }
/// }
/// ```
#[must_use = "decode with the selected reference, or drop to cancel"]
pub struct PreparedClientState<'owner, 'reader, 'body> {
    pub(super) owner: &'owner mut ClientStateDecoder,
    pub(super) reader: &'reader mut BodyReader<'body>,
    pub(super) candidate: BodyReader<'body>,
    pub(super) indices: LegacyIndexDecoder,
    pub(super) number: u32,
}
impl PreparedClientState<'_, '_, '_> {
    /// Returns the validated wire index for caller-owned prior snapshot lookup.
    #[must_use]
    pub fn number(&self) -> u32 {
        self.number
    }
    /// Decodes an owned record, then commits both reader/index owners together.
    /// `None` means zero words; client-state records never consult entity baselines.
    /// This does not admit a complete snapshot, update identities or acknowledge bytes.
    ///
    /// # Errors
    /// Rejects reference mismatch, invalid counts, truncation or unusable compact
    /// float references. Every failure preserves both owners and cached unread bits.
    pub fn decode(
        mut self,
        reference: Option<&ClientState>,
    ) -> Result<ClientStateDelta, ClientStateError> {
        if reference.is_some_and(|state| state.number() != self.number) {
            return Err(ClientStateError::ReferenceNumber);
        }
        let result = record::read(&mut self.candidate, self.number, reference)?;
        self.owner.indices = self.indices;
        *self.reader = self.candidate;
        Ok(result)
    }
}
impl fmt::Debug for PreparedClientState<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedClientState")
            .field("number", &self.number)
            .finish_non_exhaustive()
    }
}
