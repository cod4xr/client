// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned native player-state array sections with complete-section reader rollback.
//! These are wire projections, not engine storage limits or whole player-state
//! admission. Stats/ammunition precede objectives/HUD; weapon models follow HUD.

mod ammunition;
mod error;
mod groups;
mod models;
mod stats;

pub use ammunition::PlayerAmmunition;
pub use error::PlayerArrayError;
pub use models::WeaponModels;
pub use stats::PlayerStats;

/// Native mixed-width stat slots; semantic gameplay interpretation is separate.
pub const PLAYER_STAT_SLOTS: usize = 5;
/// Stored-ammunition entries updated by the native wire, despite 128 native C slots.
/// Higher engine-owned entries are outside this projection and must not be cleared.
pub const STORED_AMMO_SLOTS: usize = 64;
/// Clip-ammunition entries updated by the native wire.
pub const CLIP_AMMO_SLOTS: usize = 128;
/// Raw per-weapon model bytes in the native wire table.
pub const WEAPON_MODEL_SLOTS: usize = 128;
