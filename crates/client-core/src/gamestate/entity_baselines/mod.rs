// SPDX-License-Identifier: AGPL-3.0-or-later
//! Protocol-21 initial entity baselines, decoded from a zero reference state.
//! These inert wire values are not validated engine objects or snapshot deltas.

mod decoder;
#[cfg(test)]
mod delta_field_cases;
pub mod deltas;
mod error;
#[cfg(test)]
mod field_cases;
mod field_error;
mod field_values;
mod fields;
mod state;
mod tables;
mod word;

pub use decoder::EntityBaselineDecoder;
pub use error::EntityBaselineError;
pub use state::EntityBaseline;
pub use word::BaselineWord;

/// Fixed words in one baseline representation, excluding the entity number.
/// This is not an engine entity-count or object-size allocation limit.
pub const BASELINE_ENTITY_WORDS: usize = 60;
