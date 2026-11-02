// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete ordinary gamestate service carriers before world/ACK/engine application.
//! Commands and identities retain their ordering across one complete gamestate.
mod decode;
mod error;
mod plan;
mod storage;

pub use error::GameStateServiceError;
pub use plan::{GameStateServicePlan, GameStateServicePrefix};
pub use storage::GameStateServiceStorage;
