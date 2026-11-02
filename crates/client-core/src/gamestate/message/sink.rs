// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Counts, GameStateStorage};
use crate::{
    client_data::ClientData,
    gamestate::{configstrings::ConfigStringSlot, entity_baselines::EntityBaseline},
};

// Private typed sinks share one wire walker. No external callback can observe partial
// validation. Writer is used only after a complete identical Noop walk succeeds.
pub(super) trait Sink {
    fn configstring(&mut self, index: u32, bytes: &[u8]);
    fn entity(&mut self, value: EntityBaseline);
    fn client(&mut self, value: ClientData);
}

pub(super) struct Noop;
impl Sink for Noop {
    fn configstring(&mut self, _index: u32, _bytes: &[u8]) {}
    fn entity(&mut self, _value: EntityBaseline) {}
    fn client(&mut self, _value: ClientData) {}
}

pub(super) struct Writer<'writer, 'storage> {
    pub storage: &'writer mut GameStateStorage<'storage>,
    pub written: Counts,
}

impl Sink for Writer<'_, '_> {
    fn configstring(&mut self, index: u32, bytes: &[u8]) {
        let offset = self.written.text_bytes;
        let end = offset + bytes.len();
        self.storage.configstring_bytes[offset..end].copy_from_slice(bytes);
        self.storage.configstring_slots[self.written.configstrings] = ConfigStringSlot {
            index,
            offset,
            length: bytes.len(),
        };
        self.written.text_bytes = end;
        self.written.configstrings += 1;
    }
    fn entity(&mut self, value: EntityBaseline) {
        self.storage.entity_slots[self.written.entities].value = Some(value);
        self.written.entities += 1;
    }
    fn client(&mut self, value: ClientData) {
        self.storage.client_slots[self.written.clients].value = Some(value);
        self.written.clients += 1;
    }
}
