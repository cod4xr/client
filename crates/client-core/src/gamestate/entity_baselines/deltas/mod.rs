// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned explicit-profile entity updates against a caller-selected prior state.
//! This is a record codec, not snapshot framing, stream termination or engine state.

mod decoder;
mod error;
mod prepared;
mod record;
mod state;

/// Shared semantic word names; no native memory layout or union casts are exposed.
pub use super::BaselineWord as EntityWord;
pub use decoder::EntityDeltaDecoder;
pub use error::EntityDeltaError;
pub use prepared::PreparedEntityDelta;
pub use state::{EntityDelta, EntityState};
