// SPDX-License-Identifier: AGPL-3.0-or-later
//! Transactional protocol-21 identity state after complete message validation.
//! Names remain inert data; peer binding, engine application and ACKs belong to the host.
mod baseline;
mod batch;
mod error;
mod prepared;
mod slot;
mod stage;
mod state;
mod updates;

pub use error::ClientIdentityError;
pub use prepared::PreparedClientIdentities;
pub use slot::ClientIdentitySlot;
pub use state::ClientIdentityState;

/// Independent host inspection/retention policy, without a fixed global player table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientIdentityBudget {
    /// Maximum distinct retained identities; baseline wire identifiers stay separate.
    pub max_clients: usize,
    /// Maximum inspected records in one baseline or update batch, including ignored records.
    pub max_records: usize,
}
/// Proposed/committed identity metadata, without names or authentication authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientIdentityChange {
    /// Local baseline generation; checked increment occurs only for a new baseline.
    pub generation: u64,
    /// True for a replacement gamestate baseline, false for an ordinary update batch.
    pub baseline_replaced: bool,
    /// Inspected input records, including duplicates and non-next updates.
    pub records: usize,
    /// Baseline records or exact-next updates accepted into the proposed state.
    pub accepted_records: usize,
    /// Ordinary updates ignored because their sequence was not the exact successor.
    pub ignored_records: usize,
    /// Distinct retained identities after this transaction.
    pub clients: usize,
    /// Proposed accepted config-data sequence, preserving all thirty-two wire bits.
    pub sequence: u32,
    /// Own client index copied from the validated gamestate trailer.
    pub own_client_index: u32,
}
