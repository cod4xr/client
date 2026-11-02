// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{OBJECTIVE_SLOTS, ObjectiveError, ObjectiveState, decode};
use crate::{BaselineProtocol, message_body::BodyReader};
use std::fmt;

/// Owned complete native objective section before gameplay/resource interpretation.
/// Fixed wire slots require no peer-selected allocation and retain no message borrow.
#[derive(Clone, Eq, PartialEq)]
pub struct SnapshotObjectives {
    profile: BaselineProtocol,
    objectives: [ObjectiveState; OBJECTIVE_SLOTS],
}
impl SnapshotObjectives {
    /// Constructs the explicit zero reference used when no prior snapshot exists.
    #[must_use]
    pub fn zero(profile: BaselineProtocol) -> Self {
        Self {
            profile,
            objectives: std::array::from_fn(|_| ObjectiveState::default()),
        }
    }
    /// Decodes the complete section into privately owned words before cursor commit.
    /// `None` supplies zero state; unchanged sections/fields retain the full prior.
    /// Both reviewed profiles use this same schema, without legacy coordinate bias.
    /// This does not admit the containing player state, snapshot or message ACK.
    ///
    /// # Errors
    /// Rejects profile mismatch, invalid compact-float references and truncation.
    /// Every failure preserves the original byte cursor, cached bits and prior set.
    pub fn parse(
        reader: &mut BodyReader<'_>,
        profile: BaselineProtocol,
        reference: Option<&Self>,
    ) -> Result<Self, ObjectiveError> {
        if reference.is_some_and(|prior| prior.profile != profile) {
            return Err(ObjectiveError::ReferenceProfile);
        }
        let mut result = reference.cloned().unwrap_or_else(|| Self::zero(profile));
        let mut candidate = reader.clone();
        decode::read(&mut candidate, &mut result.objectives)?;
        *reader = candidate;
        Ok(result)
    }
    /// Returns the explicit baseline profile used for this section.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Borrows one inert native wire slot, returning `None` outside this schema.
    #[must_use]
    pub fn get(&self, slot: usize) -> Option<&ObjectiveState> {
        self.objectives.get(slot)
    }
    /// Iterates all sixteen wire slots, including empty or unrecognized status values.
    #[must_use]
    pub fn objectives(&self) -> impl ExactSizeIterator<Item = &ObjectiveState> {
        self.objectives.iter()
    }
}
impl fmt::Debug for SnapshotObjectives {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotObjectives")
            .field("profile", &self.profile)
            .field("slots", &self.objectives.len())
            .finish_non_exhaustive()
    }
}
