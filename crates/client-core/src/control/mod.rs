// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned baseline control/movement frames, before channel I/O and input admission.

mod assemble;
mod error;
mod header;
mod movement;
mod plain;
mod plan;
mod server_key;

pub use error::ControlMessageError;
pub use header::ClientControlHeader;
pub use movement::{MAX_PACKET_USER_COMMANDS, MovementBatch, MovementMessageError};
pub use plan::ControlMessagePlan;
pub use server_key::ServerCommandKey;

/// Native client control-frame budget, independent of engine/custom-profile memory.
pub const MAX_CONTROL_MESSAGE_BYTES: usize = 2048;
