// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{PLAYER_SCALAR_WORDS, PlayerWord};
use crate::BaselineProtocol;
use std::fmt;

/// Owned inert scalar projection, without prediction, arrays or engine authority.
#[derive(Clone, Eq, PartialEq)]
pub struct PlayerScalars {
    pub(super) profile: BaselineProtocol,
    pub(super) words: [u32; PLAYER_SCALAR_WORDS],
}
impl PlayerScalars {
    /// Constructs an explicit zero reference for a reviewed baseline profile.
    #[must_use]
    pub fn zero(profile: BaselineProtocol) -> Self {
        match profile {
            BaselineProtocol::Legacy6 | BaselineProtocol::Cod4x21 => Self {
                profile,
                words: [0; PLAYER_SCALAR_WORDS],
            },
        }
    }
    /// Returns the caller-selected baseline profile, independent of discovery.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns one complete inert word by a reviewed semantic field key.
    #[must_use]
    pub fn word(&self, word: PlayerWord) -> u32 {
        self.words[word.index()]
    }
    pub(in crate::entities) fn replace_word(&mut self, word: PlayerWord, value: u32) {
        self.words[word.index()] = value;
    }
}
impl fmt::Debug for PlayerScalars {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlayerScalars")
            .field("profile", &self.profile)
            .field("words", &self.words.len())
            .finish_non_exhaustive()
    }
}
