// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller-funded coherent snapshot retention, independent of native fixed rings.
//! Storage commits are separate from transport receipt and enclosing-message ACKs.

mod error;
mod lookup;
mod owner;
mod prepared;
mod slot;

pub use error::SnapshotHistoryError;
pub use owner::SnapshotHistory;
pub use prepared::{PreparedSnapshotRetention, SnapshotRetention};
pub use slot::SnapshotHistorySlot;
