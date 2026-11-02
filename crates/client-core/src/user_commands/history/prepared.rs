// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{InputCommandHistory, InputCommandId, InputHistoryError};
use crate::{
    player_state::PredictionSample,
    snapshots::SnapshotSource,
    user_commands::{MAX_PACKET_USER_COMMANDS, MovementBatch, UserCommandInput},
};
use std::fmt;

/// Bounded local transmission metadata, independent of native processing/ACKs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputTransmission {
    /// First retained input included, possibly a previously sent backup.
    pub first: InputCommandId,
    /// Last retained input included in this packet.
    pub last: InputCommandId,
    /// Resulting transmitted prefix; backup-only packets do not regress it.
    pub through: InputCommandId,
    /// Complete input count included in the prepared range.
    pub command_count: usize,
    /// Previously pending inputs newly admitted on commitment.
    pub newly_transmitted: usize,
}
/// Exclusive original input-owner lease with a complete copied native packet range.
/// Commit only after actual complete I/O; drop cancels transmission metadata.
///
/// ```compile_fail,E0499
/// use cod4xr_client_core::user_commands::{UserCommandInput,
///     history::{InputCommandHistory, InputCommandId}};
/// fn original_owner(history: &mut InputCommandHistory<'_>, id: InputCommandId, input: UserCommandInput) {
///     let prepared = history.prepare_range(id, id).unwrap();
///     history.enqueue(input).unwrap();
///     let _ = prepared.commit_transmission();
/// }
/// ```
#[must_use = "Commit only after complete I/O, or drop to cancel transmission admission."]
pub struct PreparedInputBatch<'owner, 'storage> {
    pub(super) owner: &'owner mut InputCommandHistory<'storage>,
    pub(super) commands: [UserCommandInput; MAX_PACKET_USER_COMMANDS],
    pub(super) summary: InputTransmission,
    pub(super) pending_after: usize,
}
impl PreparedInputBatch<'_, '_> {
    /// Borrows the exact complete command prefix without exposing unused copied cells.
    #[must_use]
    pub fn commands(&self) -> &[UserCommandInput] {
        &self.commands[..self.summary.command_count]
    }
    /// Returns prospective bounded metadata without changing the original owner.
    #[must_use]
    pub fn summary(&self) -> InputTransmission {
        self.summary
    }
    /// Binds a movement request to this original input world's profile/epoch.
    /// The packet builder separately checks header/profile/message relationships.
    /// Prediction/report time remains explicit; no input-time matching is invented.
    ///
    /// # Errors
    /// Mismatched prediction profile or snapshot world/profile leaves the lease and
    /// original owner unchanged. No packet building or I/O occurs here.
    pub fn movement<'snapshot>(
        &self,
        checksum_feed: u32,
        prediction: PredictionSample,
        snapshot: Option<SnapshotSource<'snapshot>>,
    ) -> Result<MovementBatch<'_, 'snapshot>, InputHistoryError> {
        if prediction.profile() != self.owner.profile {
            return Err(InputHistoryError::PredictionProfile);
        }
        if snapshot.is_some_and(|source| {
            source.profile() != self.owner.profile || source.message().epoch() != self.owner.epoch
        }) {
            return Err(InputHistoryError::SnapshotBinding);
        }
        Ok(MovementBatch {
            commands: self.commands(),
            checksum_feed,
            prediction,
            snapshot,
        })
    }
    /// Admits this exact original range after the host observed complete frame I/O.
    /// Performs only preflighted counter writes, without allocation or fallible work.
    /// It does not retire input, interpret a reliable ACK or claim server processing.
    #[must_use]
    pub fn commit_transmission(self) -> InputTransmission {
        self.owner.transmitted = self.summary.through.sequence;
        self.owner.pending = self.pending_after;
        self.summary
    }
}
impl fmt::Debug for PreparedInputBatch<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedInputBatch")
            .field("commands", &self.summary.command_count)
            .field("new_commands", &self.summary.newly_transmitted)
            .finish_non_exhaustive()
    }
}
