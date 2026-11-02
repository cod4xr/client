// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{gamestate::GameStateStorage, service_controls::ClientServiceStorage};
use std::fmt;

/// Independent private scratch and six final output areas for an ordinary carrier.
/// Every final byte/slot stays unchanged on any profile, framing or budget failure.
pub struct GameStateServiceStorage<'scratch, 'output> {
    /// Private normalized outer command string workspace, unspecified on failure.
    pub command_scratch: &'scratch mut [u8],
    /// Private inner configstring workspace, unspecified on failure.
    pub configstring_scratch: &'scratch mut [u8],
    /// Outer command bytes, identity scratch and final sequenced identity updates.
    pub controls: ClientServiceStorage<'scratch, 'output>,
    /// Inner configstring bytes/metadata, entity baselines and gamestate client records.
    pub game_state: GameStateStorage<'output>,
}
impl fmt::Debug for GameStateServiceStorage<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameStateServiceStorage")
            .field("command_scratch_bytes", &self.command_scratch.len())
            .field(
                "configstring_scratch_bytes",
                &self.configstring_scratch.len(),
            )
            .field("command_output_bytes", &self.controls.command_bytes.len())
            .field(
                "identity_scratch_slots",
                &self.controls.client_scratch.len(),
            )
            .field("identity_output_slots", &self.controls.client_updates.len())
            .field("game_state", &self.game_state)
            .finish_non_exhaustive()
    }
}
