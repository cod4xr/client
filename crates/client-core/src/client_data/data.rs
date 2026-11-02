// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BASELINE_CLIENT_SLOTS, ClientDataError, MAX_BASELINE_CLAN_TAG_BYTES,
    MAX_BASELINE_CLIENT_NAME_BYTES, require_profile,
};
use crate::{BaselineProtocol, message_body::BodyReader};
use cod4x_protocol::MessageStringLimit;
use std::fmt;

/// One owned, normalized baseline identity, independent of its input frame.
/// Fixed arrays are wire-field capacities, never a player table or engine limit.
#[derive(Clone, Eq, PartialEq)]
pub struct ClientData {
    index: u32,
    name: [u8; MAX_BASELINE_CLIENT_NAME_BYTES],
    name_length: usize,
    clan_tag: [u8; MAX_BASELINE_CLAN_TAG_BYTES],
    clan_tag_length: usize,
}

impl ClientData {
    /// Parses immediately after a gamestate `svc_configclient` opcode.
    /// Publishes a complete owned record and advances the reader only on success.
    /// Values remain raw bytes, with shared percent normalization and no UTF-8 guess.
    ///
    /// # Errors
    /// Rejects other profiles, out-of-range identifiers, truncation and oversized
    /// fields. Unlike native display truncation, oversized text is rejected atomically.
    pub fn parse(
        reader: &mut BodyReader<'_>,
        profile: BaselineProtocol,
    ) -> Result<Self, ClientDataError> {
        require_profile(profile)?;
        let mut candidate = reader.clone();
        let data = Self::read(&mut candidate)?;
        *reader = candidate;
        Ok(data)
    }

    pub(super) fn read(reader: &mut BodyReader<'_>) -> Result<Self, ClientDataError> {
        let index = u32::from(reader.read_u8()?);
        if index >= BASELINE_CLIENT_SLOTS {
            return Err(ClientDataError::ClientIndex);
        }
        let mut name = [0; MAX_BASELINE_CLIENT_NAME_BYTES];
        let name_length = reader.read_legacy_string(&mut name, MessageStringLimit::Normal)?;
        let mut clan_tag = [0; MAX_BASELINE_CLAN_TAG_BYTES];
        let clan_tag_length =
            reader.read_legacy_string(&mut clan_tag, MessageStringLimit::Normal)?;
        Ok(Self {
            index,
            name,
            name_length,
            clan_tag,
            clan_tag_length,
        })
    }

    /// Returns the checked baseline player identifier without narrowing engine storage.
    #[must_use]
    pub fn index(&self) -> u32 {
        self.index
    }
    /// Returns normalized raw name bytes, excluding NUL and unused array capacity.
    #[must_use]
    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_length]
    }
    /// Returns normalized raw clan-tag bytes, excluding NUL and unused array capacity.
    #[must_use]
    pub fn clan_tag(&self) -> &[u8] {
        &self.clan_tag[..self.clan_tag_length]
    }
}

impl fmt::Debug for ClientData {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientData")
            .field("index", &self.index)
            .field("name_bytes", &self.name_length)
            .field("clan_tag_bytes", &self.clan_tag_length)
            .finish_non_exhaustive()
    }
}
