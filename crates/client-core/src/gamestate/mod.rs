// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed gamestate constituents, before whole-message validation and engine application.
//! Entity baselines, constant configstring defaults and full dispatch remain separate.

pub mod configstrings;
pub mod entity_baselines;

mod message;
pub use message::{
    ClientDataSlot, EntityBaselineSlot, GameStateBudget, GameStateError, GameStatePlan,
    GameStateStorage, ReliableGameStateError, ReliableGameStatePlan,
};
