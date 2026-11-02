// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete live player-state wire projection with explicit prediction resolution.
//! Preparation owns decoded sections but retains an exclusive reader lease until
//! commitment. Session binding, typed gameplay and whole snapshot/ACK admission
//! remain responsibilities of the containing owner.

mod decode;
mod error;
pub mod history;
mod pending;
mod prediction;
mod prepared;
mod state;
mod words;

pub use error::PlayerStateError;
pub use prediction::{PredictionProvenance, PredictionRequest, PredictionSample};
pub use prepared::PreparedPlayerState;
pub use state::LivePlayerState;
pub use words::PredictionWords;

pub(crate) use pending::PendingPlayerState;
