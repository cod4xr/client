// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete native snapshot entity substreams, not whole snapshots or message ACKs.
//! Caller-funded sources and final cells replace native global entity/ring arrays.

mod error;
mod plan;
mod sink;
mod source;
mod storage;
mod walk;

pub use error::EntityStreamError;
pub use plan::SnapshotEntityPlan;
pub use source::EntityStateSource;
pub use storage::SnapshotEntitySlot;

/// Reserved ten-bit native entity number terminating this substream.
/// It is a wire marker for the reviewed profiles, not an engine allocation ceiling.
pub const ENTITY_STREAM_TERMINATOR: u32 = 1023;

/// Borrowed inert sources, validated for order/namespace/budgets before stream reads.
/// The host still binds their gamestate generation, snapshot sequence and connection.
#[derive(Clone, Copy, Debug)]
pub struct EntityStreamSources<'source> {
    /// Prior snapshot entities; reserved terminator numbers are not entities here.
    pub previous: EntityStateSource<'source>,
    /// Initial gamestate baselines used only for numbers absent from the prior snapshot.
    /// A missing baseline means an explicit zero reference, matching native storage.
    pub baselines: EntityStateSource<'source>,
}

/// Independent host work/retention bounds; no allocation follows peer identifiers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntityStreamBudget {
    /// Inspected wire indices, including the terminating index.
    pub max_wire_records: usize,
    /// Maximum prior snapshot entities inspected, including retained unchanged values.
    pub max_previous_entities: usize,
    /// Maximum initial baseline entities inspected, including unused entries.
    pub max_baselines: usize,
    /// Maximum merged output entities retained, independently of update count.
    pub max_entities: usize,
}

/// Bounded-category evidence suitable for host counters, without state words.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EntityStreamStats {
    /// Inspected indices, including the terminating index.
    pub wire_records: usize,
    /// Explicit update/removal records, excluding the terminator.
    pub updates: usize,
    /// Explicit removals, including removal of a number absent from the prior snapshot.
    pub removals: usize,
    /// Unmentioned prior entities copied unchanged, including the tail after terminator.
    pub unchanged_entities: usize,
    /// New-number updates decoded against an existing initial baseline.
    pub baseline_references: usize,
    /// New-number updates decoded against zero when no initial baseline exists.
    pub zero_references: usize,
    /// Complete merged entities retained after removals.
    pub output_entities: usize,
}
