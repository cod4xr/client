// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    ConfigMergeSlot, ConfigPatch, ConfigStateBudget, ConfigStateError, ConfigStateStats,
    ConfigValue, slot::Kind,
};

pub(super) fn capture(
    defaults: &[ConfigValue<'_>],
    patches: &[ConfigPatch<'_>],
    budget: ConfigStateBudget,
    scratch: &mut [ConfigMergeSlot],
) -> Result<ConfigStateStats, ConfigStateError> {
    let count = defaults
        .len()
        .checked_add(patches.len())
        .filter(|&value| value <= budget.max_input_records)
        .ok_or(ConfigStateError::InputRecordBudget)?;
    if count > scratch.len() {
        return Err(ConfigStateError::ScratchCapacity);
    }
    let mut stats = ConfigStateStats {
        input_records: count,
        ..ConfigStateStats::default()
    };
    for value in defaults
        .iter()
        .copied()
        .chain(patches.iter().map(|patch| patch.value))
    {
        stats.input_bytes = stats
            .input_bytes
            .checked_add(value.bytes.len())
            .filter(|&value| value <= budget.max_input_bytes)
            .ok_or(ConfigStateError::InputByteBudget)?;
    }
    let mut previous = None;
    for (position, value) in defaults.iter().enumerate() {
        if previous.is_some_and(|index| value.index <= index) {
            return Err(ConfigStateError::DefaultOrder);
        }
        if value.bytes.contains(&0) {
            return Err(ConfigStateError::EmbeddedNul);
        }
        scratch[position] = ConfigMergeSlot {
            index: value.index,
            kind: Kind::Default,
            source_position: position,
            block: 0,
        };
        previous = Some(value.index);
    }
    previous = None;
    for (position, patch) in patches.iter().enumerate() {
        if patch.starts_block {
            stats.blocks += 1; // At most one block per preflighted source record.
            previous = None;
        }
        if stats.blocks == 0 {
            return Err(ConfigStateError::MissingBlockStart);
        }
        if previous.is_some_and(|index| patch.value.index <= index) {
            return Err(ConfigStateError::PatchOrder);
        }
        if patch.value.bytes.contains(&0) {
            return Err(ConfigStateError::EmbeddedNul);
        }
        scratch[defaults.len() + position] = ConfigMergeSlot {
            index: patch.value.index,
            kind: Kind::Patch,
            source_position: position,
            block: stats.blocks,
        };
        previous = Some(patch.value.index);
    }
    scratch[..count].sort_unstable_by_key(|record| record.index);
    Ok(stats)
}
