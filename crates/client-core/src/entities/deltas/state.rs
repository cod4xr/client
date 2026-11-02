// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::EntityBaseline;
use super::EntityWord;
use std::fmt;

/// Owned inert entity state from a baseline or a successfully decoded delta.
/// Words still require engine geometry/resource validation and snapshot/epoch binding.
#[derive(Clone, Eq, PartialEq)]
pub struct EntityState(pub(super) EntityBaseline);
impl EntityState {
    /// Copies a complete checked initial baseline without retaining its source plan.
    #[must_use]
    pub fn from_baseline(baseline: &EntityBaseline) -> Self {
        Self(baseline.clone())
    }
    /// Returns the entity number associated with this exact state.
    #[must_use]
    pub fn number(&self) -> u32 {
        self.0.number()
    }
    /// Returns one raw semantic word, without float conversion or engine effects.
    #[must_use]
    pub fn word(&self, field: EntityWord) -> u32 {
        self.0.word(field)
    }
}
impl fmt::Debug for EntityState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntityState")
            .field("number", &self.number())
            .finish_non_exhaustive()
    }
}
/// One complete explicit state update or removal; neither changes host storage.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    clippy::large_enum_variant,
    reason = "The fixed wire state stays owned and allocation free; callers budget record counts."
)]
pub enum EntityDelta {
    /// A complete owned state, including retained words outside the changed prefix.
    State(EntityState),
    /// The wire record removes this entity number from the caller's chosen snapshot.
    Removed {
        /// Original checked wire number; a missing host entity is caller policy.
        number: u32,
    },
}
