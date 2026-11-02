// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{CLIP_AMMO_SLOTS, PlayerArrayError, STORED_AMMO_SLOTS, groups};
use crate::{BaselineProtocol, message_body::BodyReader};
use std::fmt;

/// Owned native ammunition wire projection; it grants no inventory/gameplay authority.
/// The native C struct has 128 stored slots, but this wire updates only its first 64.
/// Applying this projection must preserve higher engine-owned stored entries.
#[derive(Clone, Eq, PartialEq)]
pub struct PlayerAmmunition {
    profile: BaselineProtocol,
    stored: [i32; STORED_AMMO_SLOTS],
    clips: [i32; CLIP_AMMO_SLOTS],
}
impl PlayerAmmunition {
    /// Constructs an explicit zero reference for these wire-visible entries only.
    #[must_use]
    pub fn zero(profile: BaselineProtocol) -> Self {
        Self {
            profile,
            stored: [0; STORED_AMMO_SLOTS],
            clips: [0; CLIP_AMMO_SLOTS],
        }
    }
    /// Decodes stored and clip sections together before reader publication.
    /// Stored ammo has one overall change bit and four optional sixteen-entry groups;
    /// clips have eight group bits without an overall change bit. Set mask bits carry
    /// absolute signed shorts. Every omitted value retains its complete prior word.
    ///
    /// # Errors
    /// Rejects profile mismatch or truncation anywhere, including late clip groups,
    /// without changing either reader cursor or the immutable prior projection.
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
            groups::read(&mut candidate, &mut result.stored)?;
        }
        groups::read(&mut candidate, &mut result.clips)?;
        *reader = candidate;
        Ok(result)
    }
    /// Returns the explicitly selected baseline profile.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns stored ammunition within the native wire projection, never wider state.
    #[must_use]
    pub fn stored(&self, slot: usize) -> Option<i32> {
        self.stored.get(slot).copied()
    }
    /// Returns clip ammunition within the native wire projection.
    #[must_use]
    pub fn clip(&self, slot: usize) -> Option<i32> {
        self.clips.get(slot).copied()
    }
    /// Borrows the 64 native wire-visible stored values, without narrowing wider IDs.
    #[must_use]
    pub fn stored_values(&self) -> &[i32; STORED_AMMO_SLOTS] {
        &self.stored
    }
    /// Borrows the 128 native wire-visible clip values.
    #[must_use]
    pub fn clip_values(&self) -> &[i32; CLIP_AMMO_SLOTS] {
        &self.clips
    }
}
impl fmt::Debug for PlayerAmmunition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlayerAmmunition")
            .field("profile", &self.profile)
            .field("stored_slots", &self.stored.len())
            .field("clip_slots", &self.clips.len())
            .finish_non_exhaustive()
    }
}
