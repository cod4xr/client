// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::EntityState;
use super::SnapshotEntitySlot;

// Only closed internal sinks run during validation/replay; no caller callbacks.
pub(super) trait Sink {
    fn entity(&mut self, state: EntityState);
}
pub(super) struct Noop;
impl Sink for Noop {
    fn entity(&mut self, _: EntityState) {}
}
pub(super) struct Writer<'storage> {
    pub slots: &'storage mut [SnapshotEntitySlot],
    pub written: usize,
}
impl Sink for Writer<'_> {
    fn entity(&mut self, state: EntityState) {
        self.slots[self.written].0 = Some(state);
        self.written += 1;
    }
}
