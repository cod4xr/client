// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned inert protocol-21 client identity records for gamestate and later updates.
//! Decoding grants no sequence, engine, command or authentication authority.

mod data;
mod error;
mod update;

pub use data::ClientData;
pub use error::ClientDataError;
pub use update::ClientDataUpdate;

/// Protocol-21 player namespace, not an engine-wide player storage limit.
pub const BASELINE_CLIENT_SLOTS: u32 = 64;
/// Native client-name wire capacity excluding NUL, not a general engine string cap.
pub const MAX_BASELINE_CLIENT_NAME_BYTES: usize = 32;
/// Native clan-tag wire capacity excluding NUL, not a general engine string cap.
pub const MAX_BASELINE_CLAN_TAG_BYTES: usize = 12;

fn require_profile(profile: crate::BaselineProtocol) -> Result<(), ClientDataError> {
    if profile != crate::BaselineProtocol::Cod4x21 {
        return Err(ClientDataError::UnsupportedProfile);
    }
    Ok(())
}
