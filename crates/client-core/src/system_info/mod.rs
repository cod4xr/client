// SPDX-License-Identifier: AGPL-3.0-or-later
//! Read-only native system metadata. Remote variables never execute or configure the host.

mod error;
mod server_id;

use std::fmt;

use cod4x_protocol::{InfoLimit, InfoString};

use crate::config_state::ConfigSnapshot;

pub use error::SystemInfoError;

/// Native `CS_SYSTEMINFO` identifier shared by the reviewed baseline profiles.
pub const SYSTEM_INFO_INDEX: u32 = 1;

/// Host inspection budgets; these do not increase the baseline big-info wire bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemInfoBudget {
    /// Maximum encoded bytes to inspect, excluding a C-string terminator.
    pub max_encoded_bytes: usize,
    /// Maximum fields to inspect, including unknown and duplicate fields.
    pub max_fields: usize,
}

/// Complete validated system info, borrowed from caller-owned cooked config storage.
/// Unknown fields remain inert bytes; no local variables, paths or capabilities change.
pub struct SystemInfo<'storage> {
    info: InfoString<'storage>,
    server_id: u32,
    fields: usize,
}

impl<'storage> SystemInfo<'storage> {
    /// Validates the complete big-info string and one unambiguous server identifier.
    /// Unknown duplicate keys retain native first-match lookup semantics. The critical
    /// `sv_serverid` key must occur exactly once, matched without ASCII case sensitivity.
    ///
    /// # Errors
    /// Rejects host/wire budgets, malformed info, missing/duplicate IDs and decimal
    /// IDs outside signed-32 through unsigned-32 representations. Errors contain no text.
    pub fn parse(bytes: &'storage [u8], budget: SystemInfoBudget) -> Result<Self, SystemInfoError> {
        if bytes.len() > budget.max_encoded_bytes {
            return Err(SystemInfoError::ByteBudget);
        }
        let info = InfoString::parse(bytes, InfoLimit::Big).map_err(SystemInfoError::Info)?;
        let mut fields = 0;
        let mut server_id = None;
        for pair in info.pairs() {
            if fields >= budget.max_fields {
                return Err(SystemInfoError::FieldBudget);
            }
            fields += 1;
            if pair.key().eq_ignore_ascii_case(b"sv_serverid")
                && server_id.replace(pair.value()).is_some()
            {
                return Err(SystemInfoError::DuplicateServerId);
            }
        }
        let server_id = server_id.ok_or(SystemInfoError::MissingServerId)?;
        let server_id = server_id::parse(server_id).ok_or(SystemInfoError::InvalidServerId)?;
        Ok(Self {
            info,
            server_id,
            fields,
        })
    }

    /// Borrows the complete effective native system-info configstring from a snapshot.
    /// The returned view retains no source frame, merge workspace or parent wire plan.
    ///
    /// # Errors
    /// Rejects a missing system-info record or any error documented by [`Self::parse`].
    pub fn from_snapshot(
        snapshot: &'storage ConfigSnapshot<'_>,
        budget: SystemInfoBudget,
    ) -> Result<Self, SystemInfoError> {
        Self::parse(
            snapshot
                .value(SYSTEM_INFO_INDEX)
                .ok_or(SystemInfoError::MissingSystemInfo)?,
            budget,
        )
    }

    /// Returns all 32 identifier bits for an explicitly chosen protocol's control header.
    /// This is metadata, not proof of session admission or authority to acknowledge data.
    #[must_use]
    pub fn server_id(&self) -> u32 {
        self.server_id
    }

    /// Returns the first ASCII-case-insensitive field value, without UTF-8 conversion.
    /// Paths, commands and extension names returned here are data only.
    #[must_use]
    pub fn field(&self, key: &[u8]) -> Option<&'storage [u8]> {
        self.info.get(key)
    }

    /// Returns the number of inspected fields, including unknown duplicates.
    #[must_use]
    pub fn field_count(&self) -> usize {
        self.fields
    }
}

impl fmt::Debug for SystemInfo<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SystemInfo")
            .field("encoded_bytes", &self.info.as_bytes().len())
            .field("fields", &self.fields)
            .finish_non_exhaustive()
    }
}
