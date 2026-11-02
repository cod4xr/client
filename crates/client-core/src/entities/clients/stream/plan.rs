// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::ClientState;
use super::{
    ClientStateSource, ClientStreamBudget, ClientStreamError, ClientStreamStats,
    SnapshotClientSlot,
    sink::{Noop, Writer},
    walk::{self, Input},
};
use crate::{BaselineProtocol, message_body::BodyReader};
use std::fmt;

/// Complete native client-state substream merged into caller-owned final cells.
/// No prior/input borrows survive; complete snapshot/message admission is separate.
/// Borrowed cooked sources prevent overwriting their storage:
/// ```compile_fail
/// use cod4xr_client_core::{BaselineProtocol, message_body::BodyReader,
///     snapshot_clients::{ClientStateSource, ClientStreamBudget, ClientStreamError,
///         SnapshotClientPlan, SnapshotClientSlot}};
/// fn cannot_overwrite(reader: &mut BodyReader<'_>, profile: BaselineProtocol,
///                     budget: ClientStreamBudget, slots: &mut [SnapshotClientSlot])
///                     -> Result<(), ClientStreamError> {
///     let first = SnapshotClientPlan::parse(reader, profile,
///         ClientStateSource::from_states(&[]), budget, slots)?;
///     let source = first.as_source();
///     let _ = SnapshotClientPlan::parse(reader, profile, source, budget, slots)?;
///     Ok(())
/// }
/// ```
pub struct SnapshotClientPlan<'storage> {
    profile: BaselineProtocol,
    slots: &'storage [SnapshotClientSlot],
    stats: ClientStreamStats,
    consumed_bytes: usize,
}
impl<'storage> SnapshotClientPlan<'storage> {
    /// Checks the complete prior source and stream before publishing any final cell.
    /// A private validation pass proves the immutable replay's reads and output extent.
    /// Following fields and unused final tails remain available without bit alignment.
    ///
    /// # Errors
    /// Rejects prior budgets/order/namespace, record/retention/capacity exhaustion,
    /// invalid records and missing end framing. Every failure preserves the entire
    /// output buffer and both original reader cursors, including cached unread bits.
    pub fn parse(
        reader: &mut BodyReader<'_>,
        profile: BaselineProtocol,
        previous: ClientStateSource<'_>,
        budget: ClientStreamBudget,
        output: &'storage mut [SnapshotClientSlot],
    ) -> Result<Self, ClientStreamError> {
        previous.validate(budget.max_previous_clients)?;
        let input = Input {
            profile,
            previous,
            budget,
            capacity: output.len(),
        };
        let mut candidate = reader.clone();
        let start = candidate.byte_position();
        let stats = walk::walk(&mut candidate, input, &mut Noop)?;
        let mut replay = reader.clone();
        // Only closed private code runs between passes; immutable sources, bytes,
        // cursors, profile, budgets and capacity are identical to validated input.
        walk::walk(
            &mut replay,
            input,
            &mut Writer {
                slots: output,
                written: 0,
            },
        )?;
        let consumed_bytes = candidate.byte_position() - start;
        *reader = candidate;
        Ok(Self {
            profile,
            slots: &output[..stats.output_clients],
            stats,
            consumed_bytes,
        })
    }
    pub(crate) fn published(
        profile: BaselineProtocol,
        slots: &'storage [SnapshotClientSlot],
        stats: ClientStreamStats,
        consumed_bytes: usize,
    ) -> Self {
        Self {
            profile,
            slots,
            stats,
            consumed_bytes,
        }
    }
    /// Returns the explicitly selected baseline profile used for decoding.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns bounded framing/merge counters without private state words.
    #[must_use]
    pub fn stats(&self) -> ClientStreamStats {
        self.stats
    }
    /// Returns newly consumed physical bytes; preexisting cached bits can also advance.
    #[must_use]
    pub fn consumed_bytes(&self) -> usize {
        self.consumed_bytes
    }
    /// Borrows complete merged client states in strictly increasing wire-index order.
    pub fn clients(&self) -> impl Iterator<Item = &ClientState> {
        self.slots.iter().filter_map(|slot| slot.0.as_ref())
    }
    /// Reuses final cells as a later prior source, independently of this plan's lifetime.
    /// The next parse rechecks the entire source against its current host budgets.
    #[must_use]
    pub fn as_source(&self) -> ClientStateSource<'storage> {
        ClientStateSource::published(self.slots)
    }
}
impl fmt::Debug for SnapshotClientPlan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotClientPlan")
            .field("profile", &self.profile)
            .field("stats", &self.stats)
            .field("consumed_bytes", &self.consumed_bytes)
            .finish_non_exhaustive()
    }
}
