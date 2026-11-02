// SPDX-License-Identifier: AGPL-3.0-or-later
//! Live native player-scalar prefixes, not reconciled player states or snapshots.
//! Explicit profile/reference ownership and field catalogs replace C layout casts.

mod decode;
mod error;
mod prefix;
mod state;
mod table;
pub mod word;

pub use error::PlayerScalarError;
pub use prefix::PlayerScalarPrefix;
pub use state::PlayerScalars;
pub use word::PlayerWord;

/// Inert native scalar words, separate from array sections and engine allocation limits.
pub const PLAYER_SCALAR_WORDS: usize = 141;
