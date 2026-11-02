// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Zip32Error, read};

pub(super) struct End {
    pub entries: usize,
    pub directory_offset: usize,
    pub directory_bytes: usize,
}

pub(super) fn parse(input: &[u8]) -> Result<End, Zip32Error> {
    let last = input.len().checked_sub(22).ok_or(Zip32Error::MissingEnd)?;
    let first = last.saturating_sub(usize::from(u16::MAX));
    let mut found = None;
    // Classic comments have a 16-bit length. No whole-file signature scan is needed.
    for offset in first..=last {
        if input[offset..offset + 4] == *b"PK\x05\x06"
            && usize::from(read::short(input, offset + 20)?) == input.len() - offset - 22
            && found.replace(offset).is_some()
        {
            return Err(Zip32Error::AmbiguousEnd);
        }
    }
    let offset = found.ok_or(Zip32Error::MissingEnd)?;
    let record = read::span(input, offset, 22)?;
    let entries = read::short(record, 10)?;
    let size = read::word(record, 12)?;
    let start = read::word(record, 16)?;
    if read::short(record, 4)? != 0
        || read::short(record, 6)? != 0
        || entries == u16::MAX
        || size == u32::MAX
        || start == u32::MAX
    {
        return Err(Zip32Error::UnsupportedFeature);
    }
    if read::short(record, 8)? != entries {
        return Err(Zip32Error::Structure);
    }
    let directory_offset = read::host(start)?;
    let directory_bytes = read::host(size)?;
    if read::add(directory_offset, directory_bytes)? != offset {
        return Err(Zip32Error::Structure);
    }
    Ok(End {
        entries: usize::from(entries),
        directory_offset,
        directory_bytes,
    })
}
