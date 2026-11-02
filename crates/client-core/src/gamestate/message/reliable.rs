// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    GameStateBudget, GameStateError, GameStatePlan, GameStateStorage, ReliableGameStateError,
    sink::Noop, walk,
};
use crate::{BaselineProtocol, message_body::BodyReader};
use std::fmt;

/// Complete reviewed protocol-21 auxiliary application body containing a gamestate.
/// The transport's four-byte length prefix is excluded from this input/plan.
pub struct ReliableGameStatePlan<'storage> {
    game_state: GameStatePlan<'storage>,
    reserved_checksum_word: u32,
    body_bytes: usize,
    unused_cached_bits: u8,
}

impl<'storage> ReliableGameStatePlan<'storage> {
    /// Validates command, embedded opcode, complete gamestate and final reserved word.
    /// Exact physical exhaustion is required; unused bits in an already consumed
    /// legacy bit byte are inert native padding, never another command. No implicit
    /// defaults, resource loads, session updates or command execution occur.
    ///
    /// # Errors
    /// Rejects unsupported profiles, host budgets, malformed/truncated wrappers or
    /// constituents, extra bytes and caller capacity exhaustion before any final writes.
    /// Private string scratch is unspecified on failure.
    pub fn parse(
        body: &[u8],
        profile: BaselineProtocol,
        maximum_body_bytes: usize,
        budget: GameStateBudget,
        scratch: &mut [u8],
        storage: GameStateStorage<'storage>,
    ) -> Result<Self, ReliableGameStateError> {
        if profile != BaselineProtocol::Cod4x21 {
            return Err(GameStateError::UnsupportedProfile.into());
        }
        let mut reader = BodyReader::try_new(body, maximum_body_bytes)?;
        if reader.read_i32()? != 1 {
            return Err(ReliableGameStateError::Command);
        }
        if reader.read_u8()? != 1 {
            return Err(ReliableGameStateError::Opcode);
        }
        let start = reader.clone();
        let layout = walk::walk(
            &mut reader,
            profile,
            budget,
            storage.capacities(),
            scratch,
            &mut Noop,
        )?;
        let reserved_checksum_word = u32::from_le_bytes(reader.read_i32()?.to_le_bytes());
        if reader.remaining_bytes() != 0 {
            return Err(ReliableGameStateError::TrailingBytes);
        }
        let unused_cached_bits = reader.remaining_cached_bits();
        // Whole-body framing, including fields after gamestate, is proven before
        // the private publication pass. The original borrowed frame is not retained.
        let game_state = GameStatePlan::publish(&start, profile, budget, scratch, storage, layout)?;
        Ok(Self {
            game_state,
            reserved_checksum_word,
            body_bytes: body.len(),
            unused_cached_bits,
        })
    }
    /// Borrows the fully validated embedded gamestate, before engine/default application.
    #[must_use]
    pub fn game_state(&self) -> &GameStatePlan<'storage> {
        &self.game_state
    }
    /// Returns the opaque trailing word without activating capabilities or content trust.
    /// The pinned native writer uses zero; optional native builds interpret it separately.
    #[must_use]
    pub fn reserved_checksum_word(&self) -> u32 {
        self.reserved_checksum_word
    }
    /// Returns complete application-body bytes, excluding the transport length prefix.
    #[must_use]
    pub fn body_bytes(&self) -> usize {
        self.body_bytes
    }
}

impl fmt::Debug for ReliableGameStatePlan<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReliableGameStatePlan")
            .field("body_bytes", &self.body_bytes)
            .field("unused_cached_bits", &self.unused_cached_bits)
            .field("game_state", &self.game_state)
            .finish_non_exhaustive()
    }
}
