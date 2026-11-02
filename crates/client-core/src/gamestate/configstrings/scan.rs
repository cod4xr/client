// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    COD4X_CONFIGSTRING_SLOTS, ConfigStringBudget, ConfigStringError, LEGACY_CONFIGSTRING_SLOTS,
    MAX_GAMESTATE_CONFIGSTRING_BYTES,
};
use crate::{BaselineProtocol, message_body::BodyReader};
use cod4x_protocol::MessageStringLimit;

fn namespace(profile: BaselineProtocol) -> u32 {
    match profile {
        BaselineProtocol::Legacy6 => LEGACY_CONFIGSTRING_SLOTS,
        BaselineProtocol::Cod4x21 => COD4X_CONFIGSTRING_SLOTS,
    }
}

pub(in crate::gamestate) fn count(
    reader: &mut BodyReader<'_>,
    profile: BaselineProtocol,
    budget: ConfigStringBudget,
    capacity: usize,
) -> Result<usize, ConfigStringError> {
    let raw = match profile {
        BaselineProtocol::Legacy6 => i32::from(reader.read_i16()?),
        BaselineProtocol::Cod4x21 => reader.read_i32()?,
    };
    let count = u32::try_from(raw)
        .ok()
        .filter(|&value| value > 0 && value <= namespace(profile))
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(ConfigStringError::Count)?;
    if count > budget.max_records {
        return Err(ConfigStringError::RecordBudget);
    }
    if count > capacity {
        return Err(ConfigStringError::RecordCapacity);
    }
    Ok(count)
}

pub(in crate::gamestate) fn entry(
    reader: &mut BodyReader<'_>,
    profile: BaselineProtocol,
    previous: &mut Option<u32>,
    scratch: &mut [u8],
) -> Result<(u32, usize), ConfigStringError> {
    let raw = match profile {
        BaselineProtocol::Legacy6 => i64::from(if reader.read_bits(1)? == 1 {
            previous.map_or(0, |index| index + 1)
        } else {
            reader.read_bits(12)?
        }),
        BaselineProtocol::Cod4x21 => i64::from(reader.read_i32()?),
    };
    let index = u32::try_from(raw)
        .ok()
        .filter(|&value| value < namespace(profile) && previous.is_none_or(|old| value > old))
        .ok_or(ConfigStringError::Index)?;
    let length = reader.read_legacy_string(scratch, MessageStringLimit::Big)?;
    if length > MAX_GAMESTATE_CONFIGSTRING_BYTES {
        return Err(ConfigStringError::StringLength);
    }
    *previous = Some(index);
    Ok((index, length))
}

pub(in crate::gamestate) fn text_size(
    current: usize,
    length: usize,
    budget: ConfigStringBudget,
    capacity: usize,
) -> Result<usize, ConfigStringError> {
    let next = current
        .checked_add(length)
        .filter(|&value| value <= budget.max_text_bytes)
        .ok_or(ConfigStringError::TextBudget)?;
    if next > capacity {
        return Err(ConfigStringError::OutputCapacity);
    }
    Ok(next)
}
