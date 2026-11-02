// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    player_state::PredictionSample, snapshots::SnapshotSource, user_commands::UserCommandInput,
};
use std::fmt;

/// Explicit movement input, prediction report and optional admitted snapshot reference.
/// Construction alone grants no session, command-timing or transmission authority.
#[derive(Clone, Copy)]
pub struct MovementBatch<'commands, 'snapshot> {
    /// One through thirty-two complete native input records; never silently truncated.
    pub commands: &'commands [UserCommandInput],
    /// Original gamestate's private checksum feed, not a peer credential.
    pub checksum_feed: u32,
    /// Explicit current-profile prediction report; origin/time are sent unchanged.
    pub prediction: PredictionSample,
    /// Current-session admitted snapshot, if delta snapshots are desired. Matching
    /// message ACK selects delta; absence or an older reference selects no-delta.
    pub snapshot: Option<SnapshotSource<'snapshot>>,
}
impl fmt::Debug for MovementBatch<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MovementBatch")
            .field("commands", &self.commands.len())
            .field("profile", &self.prediction.profile())
            .field("snapshot", &self.snapshot.is_some())
            .finish_non_exhaustive()
    }
}
