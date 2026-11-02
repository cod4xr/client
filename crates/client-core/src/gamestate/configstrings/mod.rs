// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit configstring patches in a baseline `svc_configstring` gamestate block.
//! Native implicit defaults are not synthesized. Values remain inert private bytes.

mod entry;
mod error;
mod plan;
mod scan;

pub use entry::{ConfigString, ConfigStringSlot};
pub use error::ConfigStringError;
pub use plan::ConfigStringBlockPlan;

/// Protocol-6 configstring namespace; not an internal engine allocation limit.
pub const LEGACY_CONFIGSTRING_SLOTS: u32 = 2442;
/// Protocol-21 configstring namespace; not an internal engine allocation limit.
pub const COD4X_CONFIGSTRING_SLOTS: u32 = 4884;
/// Native gamestate maximum excluding NUL, stricter than the general big-string codec.
pub const MAX_GAMESTATE_CONFIGSTRING_BYTES: usize = 8190;

/// Caller-selected work and retained text budgets, independent of wire namespaces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConfigStringBudget {
    /// Maximum explicit entries inspected in this block.
    pub max_records: usize,
    /// Maximum combined normalized bytes, excluding NUL terminators.
    pub max_text_bytes: usize,
}
