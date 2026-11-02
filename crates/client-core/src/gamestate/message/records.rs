// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{ClientDataSlot, Counts, EntityBaselineSlot, GameStateStorage};
use crate::{
    BaselineProtocol,
    client_data::ClientData,
    gamestate::{
        configstrings::{ConfigString, ConfigStringSlot},
        entity_baselines::EntityBaseline,
    },
};
use std::fmt;

/// Complete cooked gamestate records, separate from profile-specific trailer words.
/// Private construction occurs only after complete constituent validation. This
/// borrowed view grants no enclosing-message, ACK, engine or resource authority.
/// Its final storage remains borrowed after the containing plan is released:
///
/// ```compile_fail,E0499
/// use cod4xr_client_core::{BaselineProtocol, message_body::BodyReader,
///     gamestate::{GameStatePlan, GameStateBudget, GameStateStorage, configstrings::ConfigStringSlot}};
/// fn original_output(reader: &mut BodyReader<'_>, bytes: &mut [u8], slots: &mut [ConfigStringSlot], budget: GameStateBudget) {
///     let records = GameStatePlan::parse(reader, BaselineProtocol::Cod4x21, budget,
///         &mut [0; 8192], GameStateStorage { configstring_bytes: &mut *bytes,
///             configstring_slots: slots, entity_slots: &mut [], client_slots: &mut [] })
///         .unwrap().records();
///     bytes.fill(0);
///     let _ = records.text_bytes();
/// }
/// ```
#[derive(Clone, Copy)]
pub struct GameStateRecords<'storage> {
    profile: BaselineProtocol,
    bytes: &'storage [u8],
    strings: &'storage [ConfigStringSlot],
    entities: &'storage [EntityBaselineSlot],
    clients: &'storage [ClientDataSlot],
    blocks: usize,
}
impl<'storage> GameStateRecords<'storage> {
    pub(super) fn published(
        profile: BaselineProtocol,
        storage: GameStateStorage<'storage>,
        counts: Counts,
    ) -> Self {
        let GameStateStorage {
            configstring_bytes,
            configstring_slots,
            entity_slots,
            client_slots,
        } = storage;
        Self {
            profile,
            bytes: &configstring_bytes[..counts.text_bytes],
            strings: &configstring_slots[..counts.configstrings],
            entities: &entity_slots[..counts.entities],
            clients: &client_slots[..counts.clients],
            blocks: counts.configstring_blocks,
        }
    }
    /// Returns the explicitly validated record profile, not discovery negotiation.
    #[must_use]
    pub fn profile(self) -> BaselineProtocol {
        self.profile
    }
    /// Returns retained explicit patches across their original block boundaries.
    #[must_use]
    pub fn configstring_count(self) -> usize {
        self.strings.len()
    }
    /// Returns native blocks whose implicit-default scopes remain distinct.
    #[must_use]
    pub fn configstring_block_count(self) -> usize {
        self.blocks
    }
    /// Returns packed normalized text bytes, excluding terminators and unused tails.
    #[must_use]
    pub fn text_bytes(self) -> usize {
        self.bytes.len()
    }
    /// Returns retained entity baseline records, independent of engine allocation.
    #[must_use]
    pub fn entity_count(self) -> usize {
        self.entities.len()
    }
    /// Returns client identity records, including repeated identifiers.
    #[must_use]
    pub fn client_record_count(self) -> usize {
        self.clients.len()
    }
    /// Iterates complete explicit patches and original native block starts.
    #[must_use]
    pub fn configstrings(self) -> impl ExactSizeIterator<Item = ConfigString<'storage>> {
        self.strings.iter().map(move |slot| ConfigString {
            index: slot.index,
            bytes: &self.bytes[slot.offset..slot.offset + slot.length],
            first_in_block: slot.first_in_block,
        })
    }
    /// Borrows owned validated baselines, excluding unused output cells.
    pub fn entities(self) -> impl Iterator<Item = &'storage EntityBaseline> {
        self.entities.iter().filter_map(|slot| slot.value.as_ref())
    }
    /// Borrows complete identity patches without applying client or sequence state.
    pub fn client_records(self) -> impl Iterator<Item = &'storage ClientData> {
        self.clients.iter().filter_map(|slot| slot.value.as_ref())
    }
}
impl fmt::Debug for GameStateRecords<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameStateRecords")
            .field("profile", &self.profile)
            .field("configstrings", &self.configstring_count())
            .field("blocks", &self.blocks)
            .field("text_bytes", &self.text_bytes())
            .field("entities", &self.entity_count())
            .field("client_records", &self.client_record_count())
            .finish_non_exhaustive()
    }
}
