// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{gamestate::GameStateError, service_controls::ServiceControlError};
use std::{error::Error, fmt};

/// Fixed ordinary gamestate carrier failures, excluding peer data and checksum keys.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameStateServiceError {
    /// This complete ordinary gamestate layout is currently reviewed for protocol 21.
    Profile,
    /// Outer EOF arrived without the required complete gamestate.
    MissingGameState,
    /// Outer framing, command/identity validation or service budgets failed.
    Controls(ServiceControlError),
    /// The complete gamestate constituent or its independent host budgets failed.
    GameState(GameStateError),
}
impl fmt::Display for GameStateServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Profile => "ordinary gamestate profile is unsupported",
            Self::MissingGameState => "ordinary carrier requires a gamestate",
            Self::Controls(_) => "ordinary gamestate service controls rejected",
            Self::GameState(_) => "ordinary gamestate constituent rejected",
        })
    }
}
impl Error for GameStateServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Controls(error) => Some(error),
            Self::GameState(error) => Some(error),
            _ => None,
        }
    }
}
