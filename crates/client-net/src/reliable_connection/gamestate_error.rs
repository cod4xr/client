// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_client_core::gamestate::ReliableGameStateError;
use std::{error::Error, fmt};

/// Fixed typed-frame preparation failures, excluding private application bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameStateFrameError {
    /// The original endpoint has terminal framing failure.
    Failed,
    /// The original connection profile has no reviewed reliable gamestate decoder.
    UnsupportedProfile,
    /// No complete frame is ready; no I/O, decode work or release occurred.
    NoFrame,
    /// Whole-body or host resource validation failed; the frame remains ready.
    Decode(ReliableGameStateError),
}

impl fmt::Display for GameStateFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Failed => "reliable connection has terminal framing failure",
            Self::UnsupportedProfile => "original connection profile has no gamestate decoder",
            Self::NoFrame => "no complete reliable frame is ready",
            Self::Decode(_) => "ready reliable gamestate body failed validation",
        })
    }
}
impl Error for GameStateFrameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decode(error) => Some(error),
            _ => None,
        }
    }
}
