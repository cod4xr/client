// SPDX-License-Identifier: AGPL-3.0-or-later
//! Closed sparse-source traversal shared by native entity and client-state streams.

pub(super) trait NumberedState {
    fn number(&self) -> u32;
}
impl NumberedState for super::deltas::EntityState {
    fn number(&self) -> u32 {
        self.number()
    }
}
impl NumberedState for super::clients::ClientState {
    fn number(&self) -> u32 {
        self.number()
    }
}
pub(super) trait OrderedSource {
    type State: NumberedState;
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<&Self::State>;
}

pub(super) fn valid_source(source: &impl OrderedSource, maximum: u32) -> bool {
    let mut previous = None;
    for index in 0..source.len() {
        let Some(state) = source.get(index) else {
            return false;
        };
        let number = state.number();
        if number > maximum || previous.is_some_and(|prior| prior >= number) {
            return false;
        }
        previous = Some(number);
    }
    true
}

// Sources are fully validated before construction; no caller callbacks or mutation.
pub(super) struct Cursor<S> {
    source: S,
    next: usize,
}
impl<S: OrderedSource> Cursor<S> {
    pub fn new(source: S) -> Self {
        Self { source, next: 0 }
    }
    pub fn before(&mut self, number: u32) -> Option<&S::State> {
        let state = self.source.get(self.next)?;
        if state.number() >= number {
            return None;
        }
        self.next += 1;
        Some(state)
    }
    pub fn matching(&mut self, number: u32) -> Option<&S::State> {
        let state = self.source.get(self.next)?;
        if state.number() != number {
            return None;
        }
        self.next += 1;
        Some(state)
    }
    pub fn remaining(&mut self) -> Option<&S::State> {
        let state = self.source.get(self.next)?;
        self.next += 1;
        Some(state)
    }
}
