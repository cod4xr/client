// SPDX-License-Identifier: AGPL-3.0-or-later
//! Snapshot-only service carriers with complete logical EOF validation.
//! Accepts NOPs, one snapshot and EOF; other services require a combined decoder.
//! This does not admit command/identity histories, a transport ACK or gameplay.

mod error;
mod payload;
mod payload_error;
mod plan;
mod prepare;
mod prepared;

pub use error::SnapshotServiceError;
pub use payload::{SnapshotPayloadPlan, SnapshotPrediction};
pub use payload_error::SnapshotPayloadError;
pub use plan::SnapshotServicePlan;
pub use prepared::PreparedSnapshotServices;
