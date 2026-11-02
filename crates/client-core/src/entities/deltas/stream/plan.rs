// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::EntityState;
use super::{
    EntityStateSource, EntityStreamBudget, EntityStreamError, EntityStreamSources,
    EntityStreamStats, SnapshotEntitySlot,
    sink::{Noop, Writer},
    walk::{self, Input},
};
use crate::{BaselineProtocol, entities::EntityDecodeContext, message_body::BodyReader};
use std::fmt;

/// Complete native entity substream merged into final caller cells after its terminator.
/// This admits neither following client/player data nor an enclosing snapshot/message.
/// Cooked source views prevent reusing their backing cells as mutable output:
/// ```compile_fail
/// use cod4xr_client_core::{entities::EntityDecodeContext, message_body::BodyReader,
///     snapshot_entities::{EntityStateSource, EntityStreamSources, EntityStreamBudget,
///         EntityStreamError, SnapshotEntityPlan, SnapshotEntitySlot}};
/// fn cannot_overwrite(reader: &mut BodyReader<'_>, context: EntityDecodeContext,
///                     budget: EntityStreamBudget, slots: &mut [SnapshotEntitySlot])
///                     -> Result<(), EntityStreamError> {
///     let empty = EntityStateSource::from_states(&[]);
///     let first = SnapshotEntityPlan::parse(reader, context, 0,
///         EntityStreamSources { previous: empty, baselines: empty }, budget, slots)?;
///     let source = first.as_source();
///     let _ = SnapshotEntityPlan::parse(reader, context, 0,
///         EntityStreamSources { previous: source, baselines: empty }, budget, slots)?;
///     Ok(())
/// }
/// ```
pub struct SnapshotEntityPlan<'storage> {
    profile: BaselineProtocol,
    slots: &'storage [SnapshotEntitySlot],
    stats: EntityStreamStats,
    consumed_bytes: usize,
}
impl<'storage> SnapshotEntityPlan<'storage> {
    pub(crate) fn retained_slots(&self) -> &[SnapshotEntitySlot] {
        self.slots
    }
    /// Validates sources and all records before publishing merged entities or advancing
    /// the original reader. Existing cached bits survive; following data is not consumed.
    /// Source/prior/baseline borrows are not retained in the returned cooked plan.
    ///
    /// # Errors
    /// Rejects source budgets/order/namespaces, wire/retention/capacity exhaustion,
    /// malformed records and a missing/truncated terminator. Every error preserves
    /// the whole final buffer and both original reader cursors. Native geometry,
    /// reference epoch/sequence and full snapshot admission remain host preflight.
    pub fn parse(
        reader: &mut BodyReader<'_>,
        context: EntityDecodeContext,
        time: u32,
        sources: EntityStreamSources<'_>,
        budget: EntityStreamBudget,
        output: &'storage mut [SnapshotEntitySlot],
    ) -> Result<Self, EntityStreamError> {
        sources
            .previous
            .validate(budget.max_previous_entities, false)?;
        sources.baselines.validate(budget.max_baselines, true)?;
        let input = Input {
            context,
            time,
            sources,
            budget,
            capacity: output.len(),
        };
        let mut candidate = reader.clone();
        let start = candidate.byte_position();
        let stats = walk::walk(&mut candidate, input, &mut Noop)?;
        // Private replay has identical immutable input, sources/context, starting
        // cursors, budgets and capacity; every read/write was proven above.
        let mut replay = reader.clone();
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
            profile: context.profile(),
            slots: &output[..stats.output_entities],
            stats,
            consumed_bytes,
        })
    }
    pub(crate) fn published(
        profile: BaselineProtocol,
        slots: &'storage [SnapshotEntitySlot],
        stats: EntityStreamStats,
        consumed_bytes: usize,
    ) -> Self {
        Self {
            profile,
            slots,
            stats,
            consumed_bytes,
        }
    }
    /// Returns the exact explicit coordinate dialect used for this substream.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns bounded update/merge evidence, without state words or source data.
    #[must_use]
    pub fn stats(&self) -> EntityStreamStats {
        self.stats
    }
    /// Returns fresh physical bytes consumed; existing cached bits can also advance.
    #[must_use]
    pub fn consumed_bytes(&self) -> usize {
        self.consumed_bytes
    }
    /// Borrows all published merged entities in strictly increasing native number order.
    pub fn entities(&self) -> impl Iterator<Item = &EntityState> {
        self.slots.iter().filter_map(|slot| slot.0.as_ref())
    }
    /// Borrows the cooked result as a source for a later explicit stream. This view
    /// retains only final cells, independently of this plan/input/prior sources.
    /// The next parse revalidates source order/namespace and current host budgets.
    #[must_use]
    pub fn as_source(&self) -> EntityStateSource<'storage> {
        EntityStateSource::published(self.slots)
    }
}
impl fmt::Debug for SnapshotEntityPlan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotEntityPlan")
            .field("profile", &self.profile)
            .field("stats", &self.stats)
            .field("consumed_bytes", &self.consumed_bytes)
            .finish_non_exhaustive()
    }
}
