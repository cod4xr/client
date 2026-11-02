// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{ClientData, ClientDataError, require_profile};
use crate::{BaselineProtocol, message_body::BodyReader};
use std::fmt;

/// One owned sequenced `svc_configclient` patch outside a gamestate.
/// Wire sequence bits are retained; no ordering, acknowledgement or application occurs.
#[derive(Clone, Eq, PartialEq)]
pub struct ClientDataUpdate {
    sequence: u32,
    data: ClientData,
}

impl ClientDataUpdate {
    /// Parses immediately after an ordinary `svc_configclient` opcode.
    /// Its 32-bit sequence precedes the same identity fields used in gamestate.
    /// The original reader advances only after all fields validate.
    ///
    /// # Errors
    /// Rejects unsupported profiles, invalid identifiers, truncation and oversized
    /// fields. Sequence admission is a separate host/session decision.
    pub fn parse(
        reader: &mut BodyReader<'_>,
        profile: BaselineProtocol,
    ) -> Result<Self, ClientDataError> {
        require_profile(profile)?;
        let mut candidate = reader.clone();
        let sequence = u32::from_le_bytes(candidate.read_i32()?.to_le_bytes());
        let data = ClientData::read(&mut candidate)?;
        *reader = candidate;
        Ok(Self { sequence, data })
    }
    /// Returns the uninterpreted native 32-bit sequence, including its high bit.
    /// This is not authority to advance a session's accepted sequence.
    #[must_use]
    pub fn sequence(&self) -> u32 {
        self.sequence
    }
    /// Borrows the complete owned identity patch.
    #[must_use]
    pub fn data(&self) -> &ClientData {
        &self.data
    }
}

impl fmt::Debug for ClientDataUpdate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientDataUpdate")
            .field("sequence", &self.sequence)
            .field("data", &self.data)
            .finish_non_exhaustive()
    }
}
