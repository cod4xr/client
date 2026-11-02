// SPDX-License-Identifier: AGPL-3.0-or-later
mod error;
mod input;
mod key;
mod prepared;

pub use error::MovementMessageError;
pub use input::MovementBatch;
pub(super) use prepared::PreparedMovement;

/// Native commands per legacy-compatible movement packet, independent of input history.
/// Wider negotiated profiles require a separate implemented wire contract.
pub const MAX_PACKET_USER_COMMANDS: usize = 32;
