// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{GameStateFrameError, ReliableConnection};
use cod4xr_client_core::{
    BaselineProtocol,
    gamestate::{GameStateBudget, GameStateStorage, ReliableGameStatePlan},
};
use std::fmt;

/// Exclusively retains the same complete frame while the host inspects cooked data.
/// Dropping cancels release/commit; decoded caller storage remains valid. Session and
/// engine state are untouched. No network I/O can replace the frame through this lease.
#[must_use = "inspect and commit the prepared frame, or drop it to leave the frame ready"]
pub struct PreparedGameStateFrame<'owner, 'network, 'output> {
    owner: &'owner mut ReliableConnection<'network>,
    plan: ReliableGameStatePlan<'output>,
}

impl<'output> PreparedGameStateFrame<'_, '_, 'output> {
    /// Borrows cooked data for bounded host/session resource preflight.
    #[must_use]
    pub fn plan(&self) -> &ReliableGameStatePlan<'output> {
        &self.plan
    }

    /// Releases exactly the prepared frame and returns its source-independent plan.
    /// Perform any fallible host/session preflight first. No engine state is applied.
    #[must_use]
    pub fn commit(self) -> ReliableGameStatePlan<'output> {
        // Construction requires a ready frame; the exclusive owner borrow prevents
        // its consumption/replacement between preparation and this infallible release.
        let _ = self.owner.consume_frame();
        self.owner.usage.committed_gamestates =
            self.owner.usage.committed_gamestates.saturating_add(1);
        self.plan
    }
}

impl<'network> ReliableConnection<'network> {
    /// Decodes the ready frame using the original profile and caller-owned output.
    /// Failure preserves final output and frame readiness. Success returns an exclusive
    /// lease: inspect the plan, then commit release or drop to cancel. Attempts/failures
    /// count validation work; only commit increments committed/released frames.
    /// This performs no I/O and does not change receipt time or transport byte ACKs.
    ///
    /// # Errors
    /// Rejects terminal framing failure, unsupported original profiles, missing frames,
    /// malformed complete bodies and exhausted host/caller capacities.
    ///
    /// A live preparation prevents replacing its frame:
    /// ```compile_fail
    /// use cod4xr_client_net::ReliableConnection;
    /// use cod4xr_client_core::gamestate::{GameStateBudget, GameStateStorage};
    /// fn cannot_replace(connection: &mut ReliableConnection<'_>, budget: GameStateBudget,
    ///                   scratch: &mut [u8], storage: GameStateStorage<'_>) {
    ///     if let Ok(prepared) = connection.prepare_game_state(1024, budget, scratch, storage) {
    ///         let _ = connection.consume_frame();
    ///         let _ = prepared.commit();
    ///     }
    /// }
    /// ```
    pub fn prepare_game_state<'owner, 'output>(
        &'owner mut self,
        maximum_body_bytes: usize,
        budget: GameStateBudget,
        scratch: &mut [u8],
        storage: GameStateStorage<'output>,
    ) -> Result<PreparedGameStateFrame<'owner, 'network, 'output>, GameStateFrameError> {
        if self.endpoint.state().failed {
            return Err(GameStateFrameError::Failed);
        }
        if self.context().summary().requested_protocol() != BaselineProtocol::Cod4x21.number() {
            return Err(GameStateFrameError::UnsupportedProfile);
        }
        let frame = self.endpoint.frame().ok_or(GameStateFrameError::NoFrame)?;
        let result = ReliableGameStatePlan::parse(
            frame.body(),
            BaselineProtocol::Cod4x21,
            maximum_body_bytes,
            budget,
            scratch,
            storage,
        );
        self.usage.gamestate_decode_attempts =
            self.usage.gamestate_decode_attempts.saturating_add(1);
        let plan = match result {
            Ok(plan) => plan,
            Err(error) => {
                self.usage.gamestate_decode_failures =
                    self.usage.gamestate_decode_failures.saturating_add(1);
                return Err(GameStateFrameError::Decode(error));
            }
        };
        Ok(PreparedGameStateFrame { owner: self, plan })
    }
}

impl fmt::Debug for PreparedGameStateFrame<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedGameStateFrame")
            .field("plan", &self.plan)
            .finish_non_exhaustive()
    }
}
