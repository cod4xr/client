// SPDX-License-Identifier: AGPL-3.0-or-later
//! Checked ordinary server-payload decoding, before service-opcode validation.
//! Decoding never commits acknowledgement, executes commands or initializes gamestate.

mod decoded;
mod decoder;
mod error;
mod mixed_snapshot_ack;
mod snapshot_ack;

pub use decoded::DecodedServerPayload;
pub use decoder::ServerPayloadDecoder;
pub use error::ServerPayloadError;
pub use mixed_snapshot_ack::{
    MixedSnapshotAcknowledgementError, MixedSnapshotRequest, PreparedMixedSnapshotAcknowledgement,
};
pub use snapshot_ack::{
    PreparedSnapshotAcknowledgement, SnapshotAcknowledgementError, SnapshotDecodeStorage,
    SnapshotPayloadRequest,
};

/// Plaintext client-command acknowledgement preceding the transformed server payload.
pub const SERVER_ACKNOWLEDGEMENT_BYTES: usize = 4;
