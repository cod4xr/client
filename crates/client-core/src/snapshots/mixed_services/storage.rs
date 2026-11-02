// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::SnapshotOutput;
use crate::service_controls::ClientServiceStorage;
use std::fmt;

/// Separate private workspaces and final control/snapshot buffers for one carrier.
pub struct MixedSnapshotStorage<'scratch, 'output> {
    /// Private bounded native-string workspace, unspecified on failure.
    pub text_scratch: &'scratch mut [u8],
    /// Private identity staging and final command/identity output; tails are preserved.
    pub controls: ClientServiceStorage<'scratch, 'output>,
    /// Final entity/client-state output; both complete buffers survive every error.
    pub snapshot: SnapshotOutput<'output>,
}
impl fmt::Debug for MixedSnapshotStorage<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MixedSnapshotStorage")
            .field("text_scratch_bytes", &self.text_scratch.len())
            .field("command_capacity", &self.controls.command_bytes.len())
            .field("identity_capacity", &self.controls.client_updates.len())
            .field("snapshot_capacity", &self.snapshot)
            .finish_non_exhaustive()
    }
}
