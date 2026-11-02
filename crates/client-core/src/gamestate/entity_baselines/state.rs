// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BASELINE_ENTITY_WORDS, BaselineWord};
use std::fmt;

/// Owned zero-reference baseline words, before engine/resource interpretation.
/// Raw floats, identifiers and unions still require typed engine validation.
#[derive(Clone, Eq, PartialEq)]
pub struct EntityBaseline {
    pub(super) number: u32,
    pub(super) words: [u32; BASELINE_ENTITY_WORDS],
}

impl EntityBaseline {
    /// Returns the shared-codec checked baseline entity number.
    #[must_use]
    pub fn number(&self) -> u32 {
        self.number
    }
    /// Returns one inert semantic word, without C layout, casts or engine authority.
    #[must_use]
    pub fn word(&self, field: BaselineWord) -> u32 {
        self.words[field as usize]
    }
}

impl fmt::Debug for EntityBaseline {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EntityBaseline")
            .field("number", &self.number)
            .field("words", &self.words.len())
            .finish_non_exhaustive()
    }
}
