// SPDX-License-Identifier: AGPL-3.0-or-later
//! Strict canonical relative file keys for virtual asset lookup, never OS path authority.
//! Native request sanitation and physical filesystem resolution are separate policies.

mod error;
mod key;

pub use error::AssetKeyError;
pub use key::AssetKey;

/// Explicit virtual-name work budgets, independent of native fixed path arrays.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssetKeyBudget {
    /// Maximum original byte length, inspected before scanning or copying.
    pub max_bytes: usize,
    /// Maximum nonempty relative components in one file key.
    pub max_components: usize,
}
