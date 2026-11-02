// SPDX-License-Identifier: AGPL-3.0-or-later
use super::GameStateError;
use crate::message_body::BodyReadError;
use std::{error::Error, fmt};

/// Whole reliable gamestate body failures, excluding private message contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReliableGameStateError {
    /// The 32-bit application command is not the reviewed gamestate command.
    Command,
    /// The embedded byte opcode is not `svc_gamestate`.
    Opcode,
    /// Extra physical bytes follow the reviewed complete wrapper/trailer.
    TrailingBytes,
    /// Whole-body host budget or shared prefix/trailer reading failed.
    Body(BodyReadError),
    /// Complete embedded gamestate or caller storage validation failed.
    GameState(GameStateError),
}

impl fmt::Display for ReliableGameStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Command => "reliable application command is not gamestate",
            Self::Opcode => "reliable gamestate embedded opcode invalid",
            Self::TrailingBytes => "reliable gamestate has trailing bytes",
            Self::Body(_) => "invalid or truncated reliable gamestate wrapper",
            Self::GameState(_) => "invalid reliable gamestate constituent",
        })
    }
}
impl Error for ReliableGameStateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            Self::GameState(error) => Some(error),
            _ => None,
        }
    }
}
impl From<BodyReadError> for ReliableGameStateError {
    fn from(value: BodyReadError) -> Self {
        Self::Body(value)
    }
}
impl From<GameStateError> for ReliableGameStateError {
    fn from(value: GameStateError) -> Self {
        Self::GameState(value)
    }
}
