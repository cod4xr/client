// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller-funded input/replay retention with local identifiers and explicit I/O admission.
//! Reliable-command ACKs do not retire inputs; reconciliation policy belongs to the host.

mod error;
mod id;
mod prepare;
mod prepared;
mod records;
mod state;
mod storage;

#[cfg(test)]
mod boundaries;

pub use error::InputHistoryError;
pub use id::InputCommandId;
pub use prepared::{InputTransmission, PreparedInputBatch};
pub use state::InputCommandHistory;
pub use storage::InputCommandSlot;
