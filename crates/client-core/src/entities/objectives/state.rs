// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{OBJECTIVE_WORDS, ObjectiveWord};
use std::fmt;

/// Owned native objective words without pointers, source borrows or resource authority.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct ObjectiveState {
    pub(super) status: u8,
    pub(super) words: [u32; OBJECTIVE_WORDS],
}
impl ObjectiveState {
    /// Returns all three raw status bits, including values requiring later validation.
    /// Unknown native enum values remain inert; they are not accepted gameplay states.
    #[must_use]
    pub fn status(&self) -> u8 {
        self.status
    }
    /// Returns a complete inert semantic word without float/resource interpretation.
    #[must_use]
    pub fn word(&self, word: ObjectiveWord) -> u32 {
        self.words[word as usize]
    }
}
impl fmt::Debug for ObjectiveState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectiveState")
            .field("words", &self.words.len())
            .finish_non_exhaustive()
    }
}
