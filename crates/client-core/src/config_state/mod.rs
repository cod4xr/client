// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller-funded sparse native default/patch merging, before engine application.
//! Block scopes are retained; source values are inert bytes, never executed or loaded.

mod capture;
mod error;
mod input;
mod select;
mod slot;
mod snapshot;

pub use error::ConfigStateError;
pub use input::{ConfigPatch, ConfigValue};
pub use slot::{ConfigMergeSlot, ConfigStateSlot};
pub use snapshot::ConfigSnapshot;

/// Host inspection and final retention budgets, independent of identifier magnitude.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConfigStateBudget {
    /// Maximum default plus explicit patch records inspected.
    pub max_input_records: usize,
    /// Maximum combined source value bytes inspected, including overridden values.
    pub max_input_bytes: usize,
    /// Maximum unique effective records retained.
    pub max_records: usize,
    /// Maximum effective packed value bytes, excluding terminators.
    pub max_text_bytes: usize,
}

/// Bounded inspection/retention evidence without values or identifiers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConfigStateStats {
    /// Inspected default and patch descriptors.
    pub input_records: usize,
    /// Inspected source value bytes, including overridden values.
    pub input_bytes: usize,
    /// Original positive native configstring blocks.
    pub blocks: usize,
    /// Unique effective records retained.
    pub records: usize,
    /// Effective packed text bytes retained.
    pub text_bytes: usize,
}
