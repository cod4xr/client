// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete native snapshot client-state substreams, distinct from identity updates.
//! Sources and output are caller-funded; no global client/history ring is required.

mod error;
mod plan;
mod sink;
mod source;
mod storage;
mod walk;

pub use error::ClientStreamError;
pub use plan::SnapshotClientPlan;
pub use source::ClientStateSource;
pub use storage::SnapshotClientSlot;

/// Independent host work and retention policy, not legacy engine storage limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientStreamBudget {
    /// Maximum inspected record headers, including the separate zero end bit.
    pub max_wire_records: usize,
    /// Maximum inspected prior states, including every unmentioned entry.
    pub max_previous_clients: usize,
    /// Maximum merged output states retained after removals.
    pub max_clients: usize,
}

/// Bounded-category counters suitable for host logging without names or state words.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ClientStreamStats {
    /// Inspected record headers, including the separate end bit.
    pub wire_records: usize,
    /// Explicit updates/removals, excluding the end bit.
    pub updates: usize,
    /// Explicit removals, including unknown prior clients.
    pub removals: usize,
    /// Unmentioned prior clients retained unchanged, including the remaining tail.
    pub unchanged_clients: usize,
    /// Updates/removals for client indices absent from the prior snapshot.
    pub zero_references: usize,
    /// Final merged states retained after removals.
    pub output_clients: usize,
}
