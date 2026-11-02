// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original-owner ACK preparation after complete specialized snapshot decoding.
mod decode;
mod error;
mod input;
mod prepared;

pub use error::SnapshotAcknowledgementError;
pub use input::{SnapshotDecodeStorage, SnapshotPayloadRequest};
pub use prepared::PreparedSnapshotAcknowledgement;
