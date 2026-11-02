// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{ClientIdentityError, ClientIdentitySlot};
use crate::{BaselineProtocol, client_data::ClientData};
use std::fmt;

/// One caller-owned identity table and accepted config-data sequence.
/// Construction grants no active baseline. Only an explicitly committed whole
/// gamestate seeds it; updates cannot start at an implicitly assumed sequence zero.
pub struct ClientIdentityState<'storage> {
    pub(super) profile: BaselineProtocol,
    pub(super) slots: &'storage mut [ClientIdentitySlot],
    pub(super) used: usize,
    pub(super) generation: u64,
    pub(super) sequence: Option<u32>,
    pub(super) own_client: Option<u32>,
}
impl<'storage> ClientIdentityState<'storage> {
    /// Borrows final storage without clearing or exposing its old unused contents.
    /// Zero capacity can admit an empty baseline; later growth needs actual capacity.
    ///
    /// # Errors
    /// Rejects profiles other than the explicit protocol-21 identity carrier.
    pub fn new(
        profile: BaselineProtocol,
        slots: &'storage mut [ClientIdentitySlot],
    ) -> Result<Self, ClientIdentityError> {
        if profile != BaselineProtocol::Cod4x21 {
            return Err(ClientIdentityError::Profile);
        }
        Ok(Self {
            profile,
            slots,
            used: 0,
            generation: 0,
            sequence: None,
            own_client: None,
        })
    }
    /// Returns distinct currently retained identities, excluding unused cells.
    #[must_use]
    pub fn len(&self) -> usize {
        self.used
    }
    /// Returns whether the current retained table is empty, independently of baseline state.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.used == 0
    }
    /// Returns a local generation, initially zero before the first baseline commit.
    /// It binds no network peer and is not an authentication credential.
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.generation
    }
    /// Returns accepted sequence bits, or `None` before baseline commit.
    #[must_use]
    pub fn accepted_sequence(&self) -> Option<u32> {
        self.sequence
    }
    /// Returns the checked own-client index, or `None` before baseline commit.
    #[must_use]
    pub fn own_client_index(&self) -> Option<u32> {
        self.own_client
    }
    /// Looks up retained inert data without allocating by identifier magnitude.
    #[must_use]
    pub fn client(&self, index: u32) -> Option<&ClientData> {
        self.slots[..self.used]
            .binary_search_by_key(&index, ClientIdentitySlot::key)
            .ok()
            .and_then(|position| self.slots[position].0.as_ref())
    }
    /// Iterates sorted unique retained records without engine/UI application.
    pub fn clients(&self) -> impl Iterator<Item = &ClientData> {
        self.slots[..self.used]
            .iter()
            .filter_map(|slot| slot.0.as_ref())
    }
}
impl fmt::Debug for ClientIdentityState<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientIdentityState")
            .field("profile", &self.profile)
            .field("clients", &self.used)
            .field("generation", &self.generation)
            .field("has_baseline", &self.sequence.is_some())
            .finish_non_exhaustive()
    }
}
