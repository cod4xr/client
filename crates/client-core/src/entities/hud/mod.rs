// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete native HUD sections with explicit coordinate context and inert owned words.
//! Wire slot/field widths are not engine UI/resource ceilings. Typed rendering,
//! reference epochs and whole player-state/snapshot admission remain separate.

mod decode;
mod error;
mod state;
mod table;
mod word;

pub use error::HudError;
pub use state::{HudElement, SnapshotHud};
pub use word::HudWord;

/// Slots in each native archival/current HUD array, not a wider engine allocation cap.
pub const HUD_SLOTS: usize = 31;
/// Semantic words in a native HUD element, independent of C struct layout.
pub const HUD_WORDS: usize = 40;
