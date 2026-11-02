// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::EntityState;
use super::{ENTITY_STREAM_TERMINATOR, EntityStreamError, SnapshotEntitySlot};
use crate::entities::ordered::{OrderedSource, valid_source};
use std::fmt;

#[derive(Clone, Copy)]
enum Source<'storage> {
    States(&'storage [EntityState]),
    Published(&'storage [SnapshotEntitySlot]),
}
/// Borrowed source with no allocation or admission authority. Stream parsing checks
/// every source entry/order/number against its role and the current host budgets.
#[derive(Clone, Copy)]
pub struct EntityStateSource<'storage>(Source<'storage>);
impl<'storage> EntityStateSource<'storage> {
    /// Borrows caller-owned inert states without assuming their order or namespace.
    #[must_use]
    pub fn from_states(states: &'storage [EntityState]) -> Self {
        Self(Source::States(states))
    }
    pub(super) fn published(slots: &'storage [SnapshotEntitySlot]) -> Self {
        Self(Source::Published(slots))
    }
    /// Returns the exact source length charged to inspection policy.
    #[must_use]
    pub fn len(self) -> usize {
        match self.0 {
            Source::States(states) => states.len(),
            Source::Published(slots) => slots.len(),
        }
    }
    /// Reports whether this borrowed source has no entries.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
    pub(super) fn get(self, index: usize) -> Option<&'storage EntityState> {
        match self.0 {
            Source::States(states) => states.get(index),
            Source::Published(slots) => slots.get(index)?.0.as_ref(),
        }
    }
    pub(crate) fn validate(self, maximum: usize, baseline: bool) -> Result<(), EntityStreamError> {
        if self.len() > maximum {
            return Err(if baseline {
                EntityStreamError::BaselineBudget
            } else {
                EntityStreamError::PreviousBudget
            });
        }
        let maximum = ENTITY_STREAM_TERMINATOR - u32::from(!baseline);
        if !valid_source(&self, maximum) {
            return Err(EntityStreamError::Source { baseline });
        }
        Ok(())
    }
    pub(super) fn lookup(self, number: u32) -> Option<&'storage EntityState> {
        let (mut low, mut high) = (0, self.len());
        while low < high {
            let middle = low + (high - low) / 2;
            let state = self.get(middle)?;
            match state.number().cmp(&number) {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => return Some(state),
            }
        }
        None
    }
}
impl OrderedSource for EntityStateSource<'_> {
    type State = EntityState;
    fn len(&self) -> usize {
        Self::len(*self)
    }
    fn get(&self, index: usize) -> Option<&EntityState> {
        Self::get(*self, index)
    }
}
impl fmt::Debug for EntityStateSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntityStateSource")
            .field("entities", &self.len())
            .finish_non_exhaustive()
    }
}
