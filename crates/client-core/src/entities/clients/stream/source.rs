// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::ClientState;
use super::{ClientStreamError, SnapshotClientSlot};
use crate::entities::ordered::{OrderedSource, valid_source};
use std::fmt;

#[derive(Clone, Copy)]
enum Source<'storage> {
    States(&'storage [ClientState]),
    Published(&'storage [SnapshotClientSlot]),
}
/// Closed borrowed source, validated for complete order/namespace/budgets on parse.
/// Snapshot sequence, connection and generation binding still belong to the host.
#[derive(Clone, Copy)]
pub struct ClientStateSource<'storage>(Source<'storage>);
impl<'storage> ClientStateSource<'storage> {
    /// Borrows inert states without assuming sorted order or snapshot admission.
    #[must_use]
    pub fn from_states(states: &'storage [ClientState]) -> Self {
        Self(Source::States(states))
    }
    pub(super) fn published(slots: &'storage [SnapshotClientSlot]) -> Self {
        Self(Source::Published(slots))
    }
    /// Returns every entry charged to source inspection policy.
    #[must_use]
    pub fn len(self) -> usize {
        match self.0 {
            Source::States(states) => states.len(),
            Source::Published(slots) => slots.len(),
        }
    }
    /// Reports whether this source has no entries.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
    pub(super) fn validate(self, maximum: usize) -> Result<(), ClientStreamError> {
        if self.len() > maximum {
            return Err(ClientStreamError::PreviousBudget);
        }
        if !valid_source(&self, 63) {
            return Err(ClientStreamError::Source);
        }
        Ok(())
    }
}
impl OrderedSource for ClientStateSource<'_> {
    type State = ClientState;
    fn len(&self) -> usize {
        Self::len(*self)
    }
    fn get(&self, index: usize) -> Option<&ClientState> {
        match self.0 {
            Source::States(states) => states.get(index),
            Source::Published(slots) => slots.get(index)?.0.as_ref(),
        }
    }
}
impl fmt::Debug for ClientStateSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientStateSource")
            .field("clients", &self.len())
            .finish_non_exhaustive()
    }
}
