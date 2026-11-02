// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{DefaultsBudget, DefaultsError, profile::Profile};
use crate::pe32::Pe32Image;

pub(super) trait Sink {
    fn record(&mut self, index: u32, bytes: &[u8]);
}
pub(super) struct Noop;
impl Sink for Noop {
    fn record(&mut self, _index: u32, _bytes: &[u8]) {}
}

pub(super) fn walk(
    image: Pe32Image<'_>,
    profile: Profile,
    budget: DefaultsBudget,
    sink: &mut impl Sink,
) -> Result<(), DefaultsError> {
    if image.image_base() != profile.image_base {
        return Err(DefaultsError::Layout);
    }
    let mut previous = 0;
    let mut text_bytes = 0;
    let mut longest = 0;
    for position in 0..=profile.records {
        let displacement = position
            .checked_mul(16)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(DefaultsError::Layout)?;
        let address = profile
            .table_address
            .checked_add(displacement)
            .ok_or(DefaultsError::Layout)?;
        let index = word(image.file_span(address, 4)?, 0);
        if position == profile.records {
            if index != 0
                || text_bytes != profile.text_bytes
                || longest != profile.longest_value
                || previous != profile.last_index
            {
                return Err(DefaultsError::Layout);
            }
            return Ok(());
        }
        if index <= previous || index >= profile.index_limit {
            return Err(DefaultsError::Layout);
        }
        let row = image.file_span(address, 16)?;
        if word(row, 8) != 0 || word(row, 12) != 0 {
            return Err(DefaultsError::Layout);
        }
        let tail = image.file_tail(word(row, 4))?;
        let limit = tail.len().min(budget.max_value_bytes.saturating_add(1));
        let length = tail[..limit]
            .iter()
            .position(|&byte| byte == 0)
            .ok_or(DefaultsError::ValueBudget)?;
        text_bytes = text_bytes
            .checked_add(length)
            .filter(|&value| value <= budget.max_text_bytes)
            .ok_or(DefaultsError::TextBudget)?;
        longest = longest.max(length);
        previous = index;
        sink.record(index, &tail[..length]);
    }
    Err(DefaultsError::Layout)
}

// Each call follows an exact checked 4/16-byte PE span and a fixed in-row offset.
fn word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}
