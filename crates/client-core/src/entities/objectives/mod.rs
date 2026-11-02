// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete native player-objective sections, independent of engine objective limits.
//! Both reviewed baseline profiles share this bounded wire schema. Typed gameplay,
//! resource lookup and containing snapshot/message admission remain separate.

mod decode;
mod error;
mod set;
mod state;
mod table;
mod word;

pub use error::ObjectiveError;
pub use set::SnapshotObjectives;
pub use state::ObjectiveState;
pub use word::ObjectiveWord;

/// Objective slots serialized by this native player-state section.
/// This wire count is not an engine-wide objective-storage allocation limit.
pub const OBJECTIVE_SLOTS: usize = 16;
/// Non-status semantic words in one native objective entry.
pub const OBJECTIVE_WORDS: usize = 6;
