// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native snapshot wire transactions and explicit caller-funded retention.
//! Explicit profiles select codecs; the host owns reference epochs and retention.

mod error;
mod header;
pub mod history;
mod input;
mod message;
pub mod mixed_services;
mod plan;
mod prediction;
mod prepare;
mod prepared;
pub mod services;
mod source;
mod transaction;

pub use error::SnapshotError;
pub use header::{SnapshotDelta, SnapshotHeader};
pub use input::{SnapshotBudget, SnapshotInput, SnapshotOutput};
pub use message::SnapshotMessage;
pub use plan::SnapshotPlan;
pub use prepared::PreparedSnapshot;
pub use source::{SnapshotBaselines, SnapshotSource};

/// Native snapshot service tag; the header parser starts immediately after it.
pub const SNAPSHOT_OPCODE: u8 = 6;
/// Physical byte fields before the native player-state bitstream.
pub const SNAPSHOT_HEADER_BYTES: usize = 6;
