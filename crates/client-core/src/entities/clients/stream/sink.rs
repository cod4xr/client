// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::ClientState;
use super::SnapshotClientSlot;

// Closed internal validation/replay sinks; no caller callbacks during publication.
pub(crate) trait Sink {
    fn client(&mut self, state: ClientState);
}
pub(crate) struct Noop;
impl Sink for Noop {
    fn client(&mut self, _: ClientState) {}
}
pub(crate) struct Writer<'storage> {
    pub slots: &'storage mut [SnapshotClientSlot],
    pub written: usize,
}
impl Sink for Writer<'_> {
    fn client(&mut self, state: ClientState) {
        self.slots[self.written].0 = Some(state);
        self.written += 1;
    }
}
