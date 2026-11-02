// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed native pre-game notices from newly admitted server commands.
//! Preparation has no session, console, filesystem, translation or UI effects.
//! Hosts preflight sibling resources before committing, then apply typed policies.

mod error;
mod event;
mod lexer;
mod prepared;
mod text;

pub use error::PreGameError;
pub use event::{PreGameAction, PreGameEvent};
pub use prepared::{CommittedPreGameCommands, PreGameBudget, PreparedPreGameCommands};
pub use text::PreGameText;
