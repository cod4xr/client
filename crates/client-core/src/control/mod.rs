// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned baseline control frames; gameplay movement and inbound decoding remain separate.

mod error;
mod header;
mod plan;
mod server_key;

pub use error::ControlMessageError;
pub use header::ClientControlHeader;
pub use plan::ControlMessagePlan;
pub use server_key::ServerCommandKey;

/// Native client control-frame budget, independent of engine/custom-profile memory.
pub const MAX_CONTROL_MESSAGE_BYTES: usize = 2048;
