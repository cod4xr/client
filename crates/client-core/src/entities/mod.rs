// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared explicit-profile entity wire formats, independent of engine object storage.
//! Baselines and deltas share value rules and native field catalogs. Whole-message
//! admission, snapshot ownership and geometry/resource interpretation remain separate.

pub mod clients;
mod context;
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
mod legacy_coordinates;
#[cfg(test)]
mod legacy_field_cases;
mod map_center;
mod state;
mod tables;
mod word;

pub use context::EntityDecodeContext;
pub use decoder::EntityBaselineDecoder;
pub use error::EntityBaselineError;
pub use map_center::LegacyMapCenter;
pub use state::EntityBaseline;
pub use word::BaselineWord;

/// Fixed words in one baseline representation, excluding the entity number.
/// This is not an engine entity-count or object-size allocation limit.
pub const BASELINE_ENTITY_WORDS: usize = 60;
