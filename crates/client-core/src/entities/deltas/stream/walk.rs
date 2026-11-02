// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{EntityDelta, EntityState, record};
use super::{
    ENTITY_STREAM_TERMINATOR, EntityStreamBudget, EntityStreamError, EntityStreamSources,
    EntityStreamStats, sink::Sink,
};
use crate::{entities::EntityDecodeContext, message_body::BodyReader};
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
    let mut cursor = 0;
    loop {
        if stats.wire_records >= input.budget.max_wire_records {
            return Err(EntityStreamError::RecordBudget);
        }
        let number = u32::from(reader.read_index(&mut indices)?);
        stats.wire_records += 1;
        if number == ENTITY_STREAM_TERMINATOR {
            break;
        }
        while let Some(previous) = input.sources.previous.get(cursor) {
            if previous.number() >= number {
                break;
            }
            emit(previous.clone(), &input, &mut stats, sink)?;
            stats.unchanged_entities += 1;
            cursor += 1;
        }
        let previous = input
            .sources
            .previous
            .get(cursor)
            .filter(|state| state.number() == number);
        let reference = if previous.is_some() {
            previous
        } else {
            input.sources.baselines.lookup(number)
        };
        if previous.is_some() {
            cursor += 1;
        } else if reference.is_some() {
            stats.baseline_references += 1;
        } else {
            stats.zero_references += 1;
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
    while let Some(previous) = input.sources.previous.get(cursor) {
        emit(previous.clone(), &input, &mut stats, sink)?;
        stats.unchanged_entities += 1;
        cursor += 1;
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
