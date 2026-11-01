// SPDX-License-Identifier: AGPL-3.0-or-later
//! Checked ordinary server-payload decoding, before service-opcode validation.
//! Decoding never commits acknowledgement, executes commands or initializes gamestate.

mod decoded;
mod decoder;
mod error;

pub use decoded::DecodedServerPayload;
pub use decoder::ServerPayloadDecoder;
pub use error::ServerPayloadError;

/// Plaintext client-command acknowledgement preceding the transformed server payload.
pub const SERVER_ACKNOWLEDGEMENT_BYTES: usize = 4;
