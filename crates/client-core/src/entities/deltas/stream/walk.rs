// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{EntityDelta, EntityState, record};
use super::{
    ENTITY_STREAM_TERMINATOR, EntityStreamBudget, EntityStreamError, EntityStreamSources,
    EntityStreamStats, sink::Sink,
};
use crate::{
    entities::{EntityDecodeContext, ordered::Cursor},
    message_body::BodyReader,
};
use cod4x_protocol::{LegacyIndexDecoder, LegacyIndexKind};

#[derive(Clone, Copy)]
pub(super) struct Input<'source> {
    pub context: EntityDecodeContext,
    pub time: u32,
    pub sources: EntityStreamSources<'source>,
    pub budget: EntityStreamBudget,
    pub capacity: usize,
}
pub(super) fn walk(
    reader: &mut BodyReader<'_>,
    input: Input<'_>,
    sink: &mut impl Sink,
) -> Result<EntityStreamStats, EntityStreamError> {
    let mut indices = LegacyIndexDecoder::new(LegacyIndexKind::Entity);
    let mut stats = EntityStreamStats::default();
    let mut cursor = Cursor::new(input.sources.previous);
    loop {
        if stats.wire_records >= input.budget.max_wire_records {
            return Err(EntityStreamError::RecordBudget);
        }
        let number = u32::from(reader.read_index(&mut indices)?);
        stats.wire_records += 1;
        if number == ENTITY_STREAM_TERMINATOR {
            break;
        }
        while let Some(previous) = cursor.before(number) {
            emit(previous.clone(), &input, &mut stats, sink)?;
            stats.unchanged_entities += 1;
        }
        let previous = cursor.matching(number);
        let reference = if previous.is_some() {
            previous
        } else {
            input.sources.baselines.lookup(number)
        };
        if previous.is_none() {
            if reference.is_some() {
                stats.baseline_references += 1;
            } else {
                stats.zero_references += 1;
            }
        }
        stats.updates += 1;
        match record::read(
            reader,
            number,
            reference,
            input.time,
            input.context.legacy_center(),
        )? {
            EntityDelta::State(entity) => emit(entity, &input, &mut stats, sink)?,
            EntityDelta::Removed { .. } => stats.removals += 1,
        }
    }
    // Native terminator retains every remaining old entity without reading fields.
    while let Some(previous) = cursor.remaining() {
        emit(previous.clone(), &input, &mut stats, sink)?;
        stats.unchanged_entities += 1;
    }
    Ok(stats)
}
fn emit(
    entity: EntityState,
    input: &Input<'_>,
    stats: &mut EntityStreamStats,
    sink: &mut impl Sink,
) -> Result<(), EntityStreamError> {
    if stats.output_entities >= input.budget.max_entities {
        return Err(EntityStreamError::EntityBudget);
    }
    if stats.output_entities >= input.capacity {
        return Err(EntityStreamError::OutputCapacity);
    }
    sink.entity(entity);
    stats.output_entities += 1;
    Ok(())
}
