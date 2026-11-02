// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hash-bound native configstring defaults from the caller's installed client image.
//! This imports data only; no executable, asset path, engine or network state is activated.

#[cfg(test)]
mod cases;
mod entry;
mod error;
mod import;
mod profile;
mod walk;

pub use entry::{ConfigDefault, DefaultSlot};
pub use error::DefaultsError;
pub use import::ImportedDefaults;
pub use profile::REVIEWED_MULTIPLAYER_SHA256;

/// Host work/storage limits, independent of game-engine arenas or legacy wire fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DefaultsBudget {
    /// Maximum complete source image bytes before hashing or metadata inspection.
    pub max_image_bytes: usize,
    /// Maximum inspected PE section headers.
    pub max_sections: u16,
    /// Maximum retained default records.
    pub max_records: usize,
    /// Maximum combined raw default bytes, excluding NULs.
    pub max_text_bytes: usize,
    /// Maximum bytes inspected in a single default value, excluding its terminator.
    pub max_value_bytes: usize,
}
