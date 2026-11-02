// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{ClientStateError, PreparedClientState};
use crate::{BaselineProtocol, message_body::BodyReader};
use cod4x_protocol::{LegacyIndexDecoder, LegacyIndexKind};
use std::fmt;

/// One explicitly selected native client-state stream, with independent index order.
/// Both reviewed baseline profiles share this schema; discovery does not select one.
pub struct ClientStateDecoder {
    pub(super) profile: BaselineProtocol,
    pub(super) indices: LegacyIndexDecoder,
    ended: bool,
}
impl ClientStateDecoder {
    /// Starts a stream at the native minus-one client index.
    #[must_use]
    pub fn new(profile: BaselineProtocol) -> Self {
        Self {
            profile,
            indices: LegacyIndexDecoder::new(LegacyIndexKind::Client),
            ended: false,
        }
    }
    /// Returns the caller's explicit wire profile, independent of server discovery.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Restarts client ordering and end-bit framing for another stream.
    pub fn reset(&mut self) {
        self.indices.reset();
        self.ended = false;
    }
    /// Reads the leading more-records bit, preparing one record or committing its end.
    /// `None` consumes exactly the zero end bit. Client index 63 is an ordinary record.
    /// `Some` leases both owners so reference lookup cannot change either one; drop
    /// cancels the entire preparation, including the leading more-records bit.
    ///
    /// # Errors
    /// Rejects ended streams, malformed indices or truncation without advancing either
    /// owner. Later reference/field failure also preserves both original owners.
    pub fn prepare<'owner, 'reader, 'body>(
        &'owner mut self,
        reader: &'reader mut BodyReader<'body>,
    ) -> Result<Option<PreparedClientState<'owner, 'reader, 'body>>, ClientStateError> {
        if self.ended {
            return Err(ClientStateError::StreamEnded);
        }
        let mut candidate = reader.clone();
        if candidate.read_bits(1)? == 0 {
            *reader = candidate;
            self.ended = true;
            return Ok(None);
        }
        let mut indices = self.indices;
        let number = u32::from(candidate.read_index(&mut indices)?);
        Ok(Some(PreparedClientState {
            owner: self,
            reader,
            candidate,
            indices,
            number,
        }))
    }
}
impl fmt::Debug for ClientStateDecoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientStateDecoder")
            .field("profile", &self.profile)
            .field("ended", &self.ended)
            .finish_non_exhaustive()
    }
}
