// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{PlayerArrayError, WEAPON_MODEL_SLOTS};
use crate::{BaselineProtocol, message_body::BodyReader};
use std::fmt;

/// Owned raw per-weapon model table, without asset lookup or resource authority.
#[derive(Clone, Eq, PartialEq)]
pub struct WeaponModels {
    profile: BaselineProtocol,
    values: [u8; WEAPON_MODEL_SLOTS],
}
impl WeaponModels {
    /// Constructs an explicit zero reference for the native table.
    #[must_use]
    pub fn zero(profile: BaselineProtocol) -> Self {
        Self {
            profile,
            values: [0; WEAPON_MODEL_SLOTS],
        }
    }
    /// Decodes one changed bit and, when set, all 128 absolute raw model bytes.
    /// Unchanged tables retain all prior bytes; `None` supplies zero state.
    ///
    /// # Errors
    /// Rejects profile mismatch or truncation without modifying the original byte
    /// cursor, cached bits or prior table. Result ownership never retains the source.
    pub fn parse(
        reader: &mut BodyReader<'_>,
        profile: BaselineProtocol,
        reference: Option<&Self>,
    ) -> Result<Self, PlayerArrayError> {
        if reference.is_some_and(|prior| prior.profile != profile) {
            return Err(PlayerArrayError::ReferenceProfile);
        }
        let mut result = reference.cloned().unwrap_or_else(|| Self::zero(profile));
        let mut candidate = reader.clone();
        if candidate.read_bits(1)? != 0 {
            candidate.read_bytes(&mut result.values)?;
        }
        *reader = candidate;
        Ok(result)
    }
    /// Returns the explicitly selected baseline profile.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns one uninterpreted model byte, or `None` outside this wire table.
    #[must_use]
    pub fn get(&self, slot: usize) -> Option<u8> {
        self.values.get(slot).copied()
    }
    /// Borrows every model byte, preserving zero and high-byte values exactly.
    #[must_use]
    pub fn values(&self) -> &[u8; WEAPON_MODEL_SLOTS] {
        &self.values
    }
}
impl fmt::Debug for WeaponModels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WeaponModels")
            .field("profile", &self.profile)
            .field("slots", &self.values.len())
            .finish_non_exhaustive()
    }
}
