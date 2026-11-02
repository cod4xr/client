// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{SnapshotInput, SnapshotOutput, SnapshotPlan};
use super::{PreparedSnapshotServices, SnapshotServiceError, prepare};
use crate::{message_body::BodyReader, service_controls::ServiceControlBudget};
use std::fmt;

/// One complete snapshot-only carrier validated through logical EOF, excluding padding.
/// NOPs may precede/follow the snapshot; other services require another decoder.
pub struct SnapshotServicePlan<'storage> {
    pub(super) snapshot: SnapshotPlan<'storage>,
    pub(super) record_count: u32,
    pub(super) consumed_bytes: usize,
}
impl<'storage> SnapshotServicePlan<'storage> {
    /// Prepares the complete service carrier without publishing its snapshot outputs.
    /// Starts at a service opcode and accepts NOPs, exactly one snapshot and EOF.
    /// It neither skips unsupported bodies nor interprets bytes after logical EOF.
    /// The caller supplies the actual containing-message/epoch association and byte
    /// budget through the existing snapshot input and body reader.
    ///
    /// # Errors
    /// Rejects exhausted budgets, missing snapshot/EOF, unsupported services and any
    /// malformed snapshot. Original reader and both complete outputs stay unchanged.
    pub fn prepare<'reader, 'body, 'source>(
        reader: &'reader mut BodyReader<'body>,
        input: SnapshotInput<'source>,
        budget: ServiceControlBudget,
        output: SnapshotOutput<'storage>,
    ) -> Result<PreparedSnapshotServices<'reader, 'body, 'source, 'storage>, SnapshotServiceError>
    {
        prepare::prepare(reader, input, budget, output)
    }
    /// Borrows the complete cooked snapshot, retaining its native body byte count.
    #[must_use]
    pub fn snapshot(&self) -> &SnapshotPlan<'storage> {
        &self.snapshot
    }
    /// Returns inspected services, including every NOP, snapshot and terminating EOF.
    #[must_use]
    pub fn record_count(&self) -> u32 {
        self.record_count
    }
    /// Returns physical bytes consumed through logical EOF, excluding later padding.
    /// Cached bits from an earlier partial byte may also have been consumed.
    #[must_use]
    pub fn consumed_bytes(&self) -> usize {
        self.consumed_bytes
    }
}
impl fmt::Debug for SnapshotServicePlan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotServicePlan")
            .field("profile", &self.snapshot.header().profile())
            .field("record_count", &self.record_count)
            .field("consumed_bytes", &self.consumed_bytes)
            .finish_non_exhaustive()
    }
}
