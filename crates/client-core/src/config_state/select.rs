// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{ConfigMergeSlot, ConfigPatch, ConfigValue, slot::Kind};

pub(super) fn winner(group: &[ConfigMergeSlot], last_block: usize) -> Option<ConfigMergeSlot> {
    if last_block == 0 {
        return None;
    } // No native block means no default reapplication.
    let mut default = None;
    let mut latest: Option<ConfigMergeSlot> = None;
    for &record in group {
        match record.kind {
            Kind::Default => default = Some(record),
            Kind::Patch => {
                if latest.is_none_or(|old| record.block > old.block) {
                    latest = Some(record);
                }
            }
        }
    }
    match (default, latest) {
        (Some(_), Some(patch)) if patch.block == last_block => Some(patch),
        (Some(default), _) => Some(default),
        (None, patch) => patch,
    }
}

pub(super) fn bytes<'source>(
    record: ConfigMergeSlot,
    defaults: &'source [ConfigValue<'_>],
    patches: &'source [ConfigPatch<'_>],
) -> &'source [u8] {
    // Every used source index was captured from enumeration of these immutable
    // slices. Caller scratch tails are excluded before sorting or lookup.
    match record.kind {
        Kind::Default => defaults[record.source_position].bytes,
        Kind::Patch => patches[record.source_position].value.bytes,
    }
}

pub(super) fn groups(records: &[ConfigMergeSlot]) -> Groups<'_> {
    Groups {
        records,
        position: 0,
    }
}
pub(super) struct Groups<'records> {
    records: &'records [ConfigMergeSlot],
    position: usize,
}
impl<'records> Iterator for Groups<'records> {
    type Item = &'records [ConfigMergeSlot];
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.records.get(self.position)?.index;
        let start = self.position;
        while self.position < self.records.len() && self.records[self.position].index == index {
            self.position += 1;
        }
        Some(&self.records[start..self.position])
    }
}
