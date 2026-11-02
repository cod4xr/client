// SPDX-License-Identifier: AGPL-3.0-or-later
use super::SnapshotError;
use cod4x_protocol::{ChannelPacket, Route};
use std::fmt;

/// Caller-supplied world epoch and containing ordinary-message association.
/// The host must change epochs across connection/map/gamestate replacement.
/// Construction does not authenticate a peer or prove message/command admission.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SnapshotMessage {
    epoch: u64,
    sequence: u32,
    command_sequence: u32,
}
impl SnapshotMessage {
    /// Associates a positive shared-codec ordinary sequence with a world epoch and
    /// the server-command sequence preceding this snapshot in its enclosing message.
    /// No time or sequence is inferred from arrival order or a native ring index.
    ///
    /// # Errors
    /// Rejects zero and sequences outside the shared ordinary-envelope range.
    pub fn new(
        epoch: u64,
        sequence: u32,
        server_command_sequence: u32,
    ) -> Result<Self, SnapshotError> {
        if sequence == 0 || ChannelPacket::try_new(sequence, Route::ToClient, None, &[]).is_err() {
            return Err(SnapshotError::MessageSequence);
        }
        Ok(Self {
            epoch,
            sequence,
            command_sequence: server_command_sequence,
        })
    }
    /// Returns the caller's explicit world epoch, not a peer credential.
    #[must_use]
    pub fn epoch(self) -> u64 {
        self.epoch
    }
    /// Returns the containing message sequence, independent of snapshot arrival count.
    #[must_use]
    pub fn sequence(self) -> u32 {
        self.sequence
    }
    /// Returns the caller-associated complete server-command sequence word.
    #[must_use]
    pub fn server_command_sequence(self) -> u32 {
        self.command_sequence
    }
}
impl fmt::Debug for SnapshotMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotMessage")
            .field("sequence", &self.sequence)
            .finish_non_exhaustive()
    }
}
