// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    ClientDataSlot, Counts, EntityBaselineSlot, GameStateBudget, GameStateError, GameStateStorage,
    Layout,
    sink::{Noop, Writer},
    walk,
};
use crate::{
    BaselineProtocol,
    client_data::ClientData,
    gamestate::{
        configstrings::{ConfigString, ConfigStringSlot},
        entity_baselines::EntityBaseline,
    },
    message_body::BodyReader,
};
use std::fmt;

/// Complete protocol-21 gamestate constituent, before engine/default application.
/// Cooked output alone is borrowed; input frames may be released after validation.
pub struct GameStatePlan<'storage> {
    profile: BaselineProtocol,
    layout: Layout,
    bytes: &'storage [u8],
    strings: &'storage [ConfigStringSlot],
    entities: &'storage [EntityBaselineSlot],
    clients: &'storage [ClientDataSlot],
}

impl<'storage> GameStatePlan<'storage> {
    /// Parses immediately after `svc_gamestate`, through its complete trailer.
    /// The enclosing service stream or reliable command envelope must still validate
    /// before session acknowledgement or engine updates. Implicit defaults are not
    /// synthesized. No strings execute and no lengths allocate storage.
    ///
    /// # Errors
    /// Rejects unsupported profiles/opcodes, malformed constituents/trailers and host
    /// work/storage exhaustion. Final storage and original reader remain unchanged;
    /// private string scratch is unspecified on failure.
    pub fn parse(
        reader: &mut BodyReader<'_>,
        profile: BaselineProtocol,
        budget: GameStateBudget,
        scratch: &mut [u8],
        storage: GameStateStorage<'storage>,
    ) -> Result<Self, GameStateError> {
        if profile != BaselineProtocol::Cod4x21 {
            return Err(GameStateError::UnsupportedProfile);
        }
        let capacities = storage.capacities();
        let mut validated = reader.clone();
        let layout = walk::walk(
            &mut validated,
            profile,
            budget,
            capacities,
            scratch,
            &mut Noop,
        )?;
        let plan = Self::publish(reader, profile, budget, scratch, storage, layout)?;
        *reader = validated;
        Ok(plan)
    }

    pub(super) fn publish(
        start: &BodyReader<'_>,
        profile: BaselineProtocol,
        budget: GameStateBudget,
        scratch: &mut [u8],
        mut storage: GameStateStorage<'storage>,
        layout: Layout,
    ) -> Result<Self, GameStateError> {
        // The same private walker sees identical immutable input, starting cursors,
        // capacities, budgets and scratch length. All reads and writes were proven.
        let mut replay = start.clone();
        walk::walk(
            &mut replay,
            profile,
            budget,
            storage.capacities(),
            scratch,
            &mut Writer {
                storage: &mut storage,
                written: Counts::default(),
            },
        )?;
        Ok(Self {
            profile,
            bytes: &storage.configstring_bytes[..layout.counts.text_bytes],
            strings: &storage.configstring_slots[..layout.counts.configstrings],
            entities: &storage.entity_slots[..layout.counts.entities],
            clients: &storage.client_slots[..layout.counts.clients],
            layout,
        })
    }
    /// Returns the explicit reviewed baseline profile.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns the server command baseline's uninterpreted 32-bit sequence.
    #[must_use]
    pub fn server_command_sequence(&self) -> u32 {
        self.layout.server_command_sequence
    }
    /// Returns the config-data baseline's uninterpreted 32-bit sequence.
    #[must_use]
    pub fn config_data_sequence(&self) -> u32 {
        self.layout.config_data_sequence
    }
    /// Returns the checked own-client wire identifier.
    #[must_use]
    pub fn client_index(&self) -> u32 {
        self.layout.client_index
    }
    /// Returns checksum feed bits for later verified content processing, never logging.
    #[must_use]
    pub fn checksum_feed(&self) -> u32 {
        self.layout.checksum_feed
    }
    /// Returns inspected service opcodes, including EOF.
    #[must_use]
    pub fn service_count(&self) -> usize {
        self.layout.counts.services
    }
    /// Returns fresh physical bytes consumed after the gamestate opcode.
    #[must_use]
    pub fn consumed_bytes(&self) -> usize {
        self.layout.consumed_bytes
    }
    /// Returns explicit patches across all blocks, preserving wire order and duplicates.
    #[must_use]
    pub fn configstring_count(&self) -> usize {
        self.strings.len()
    }
    /// Returns retained normalized text bytes, excluding NULs.
    #[must_use]
    pub fn text_bytes(&self) -> usize {
        self.bytes.len()
    }
    /// Returns entity baseline records.
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
    /// Returns client identity records, including repeated identifiers.
    #[must_use]
    pub fn client_record_count(&self) -> usize {
        self.clients.len()
    }
    /// Iterates explicit patches; native constant defaults require separate importing.
    #[must_use]
    pub fn configstrings(&self) -> impl ExactSizeIterator<Item = ConfigString<'_>> {
        self.strings.iter().map(|slot| ConfigString {
            index: slot.index,
            bytes: &self.bytes[slot.offset..slot.offset + slot.length],
        })
    }
    /// Iterates owned validated baselines without allocating an engine entity table.
    pub fn entities(&self) -> impl Iterator<Item = &EntityBaseline> {
        self.entities.iter().filter_map(|slot| slot.value.as_ref())
    }
    /// Iterates owned client patches without applying sequence or engine state.
    pub fn client_records(&self) -> impl Iterator<Item = &ClientData> {
        self.clients.iter().filter_map(|slot| slot.value.as_ref())
    }
}

impl fmt::Debug for GameStatePlan<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GameStatePlan")
            .field("services", &self.service_count())
            .field("configstrings", &self.configstring_count())
            .field("text_bytes", &self.text_bytes())
            .field("entities", &self.entity_count())
            .field("client_records", &self.client_record_count())
            .finish_non_exhaustive()
    }
}
