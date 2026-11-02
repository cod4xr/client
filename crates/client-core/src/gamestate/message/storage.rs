// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    client_data::ClientData,
    gamestate::{configstrings::ConfigStringSlot, entity_baselines::EntityBaseline},
};
use std::fmt;

/// Opaque caller-funded entity retention, initialized with `Default`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EntityBaselineSlot {
    pub(super) value: Option<EntityBaseline>,
}
/// Opaque caller-funded client identity retention, initialized with `Default`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ClientDataSlot {
    pub(super) value: Option<ClientData>,
}

/// Final caller-owned gamestate output, unchanged on any validation failure.
pub struct GameStateStorage<'storage> {
    /// Packed normalized configstring values, without terminators.
    pub configstring_bytes: &'storage mut [u8],
    /// Opaque metadata for explicit patches in their original wire order.
    pub configstring_slots: &'storage mut [ConfigStringSlot],
    /// Owned entity baseline retention.
    pub entity_slots: &'storage mut [EntityBaselineSlot],
    /// Owned client identity retention, including repeated identifiers.
    pub client_slots: &'storage mut [ClientDataSlot],
}

#[derive(Clone, Copy)]
pub(super) struct Capacities {
    pub text: usize,
    pub configstrings: usize,
    pub entities: usize,
    pub clients: usize,
}

impl GameStateStorage<'_> {
    pub(super) fn capacities(&self) -> Capacities {
        Capacities {
            text: self.configstring_bytes.len(),
            configstrings: self.configstring_slots.len(),
            entities: self.entity_slots.len(),
            clients: self.client_slots.len(),
        }
    }
}

impl fmt::Debug for GameStateStorage<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GameStateStorage")
            .field("text_capacity", &self.configstring_bytes.len())
            .field("configstring_capacity", &self.configstring_slots.len())
            .field("entity_capacity", &self.entity_slots.len())
            .field("client_capacity", &self.client_slots.len())
            .finish_non_exhaustive()
    }
}
