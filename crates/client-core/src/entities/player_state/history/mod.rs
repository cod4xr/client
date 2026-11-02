// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller-funded exact prediction retention with explicit world and replay policy.
//! Native command times are opaque wrapping keys, never ordering or ACK evidence.
//! Full storage rejects insertion; only explicit replacement/removal discards data.

mod error;
mod lookup;
mod owner;
mod prepared;
mod source;
mod storage;

pub use error::PredictionHistoryError;
pub use owner::PredictionHistory;
pub use prepared::PreparedPredictionUpdate;
pub use source::PredictionHistorySource;
pub use storage::PredictionSlot;
