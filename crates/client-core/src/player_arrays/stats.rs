// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{PLAYER_STAT_SLOTS, PlayerArrayError};
use crate::{BaselineProtocol, message_body::BodyReader};
use std::fmt;

/// Owned mixed-width native stats; slot meanings and range policy belong to gameplay.
#[derive(Clone, Eq, PartialEq)]
pub struct PlayerStats {
    profile: BaselineProtocol,
    values: [i32; PLAYER_STAT_SLOTS],
}
impl PlayerStats {
    /// Constructs an explicit zero reference without reading or allocating.
    #[must_use]
    pub fn zero(profile: BaselineProtocol) -> Self {
        Self {
            profile,
            values: [0; PLAYER_STAT_SLOTS],
        }
    }
    /// Decodes the complete native stats section before either input cursor commits.
    /// Slots 0..=2 use signed shorts, slot 3 six bits, and slot 4 an unsigned byte.
    /// Changed values are absolute; omitted slots retain their full prior word.
    ///
    /// # Errors
    /// Rejects reference-profile mismatch or truncation without changing the reader
    /// or prior state. `None` supplies zero words, not implicit engine storage.
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
            let mask = candidate.read_bits(5)?;
            for (slot, value) in result.values[..3].iter_mut().enumerate() {
                if mask & (1 << slot) != 0 {
                    *value = i32::from(candidate.read_i16()?);
                }
            }
            if mask & 8 != 0 {
                result.values[3] = candidate.read_bits(6)?.cast_signed();
            }
            if mask & 16 != 0 {
                result.values[4] = i32::from(candidate.read_u8()?);
            }
        }
        *reader = candidate;
        Ok(result)
    }
    /// Returns the explicitly selected baseline profile.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns one inert stat value or `None` outside this wire projection.
    #[must_use]
    pub fn get(&self, slot: usize) -> Option<i32> {
        self.values.get(slot).copied()
    }
    /// Borrows all five inert stat words without gameplay interpretation.
    #[must_use]
    pub fn values(&self) -> &[i32; PLAYER_STAT_SLOTS] {
        &self.values
    }
}
impl fmt::Debug for PlayerStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlayerStats")
            .field("profile", &self.profile)
            .field("slots", &self.values.len())
            .finish_non_exhaustive()
    }
}
