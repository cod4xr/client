// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Zip32Compression, Zip32EntrySlot, Zip32Error, extra, local, read};

pub(super) struct Header<'name> {
    pub version: u16,
    pub flags: u16,
    pub method: u16,
    pub crc32: u32,
    pub compressed: u32,
    pub uncompressed: u32,
    pub name: &'name [u8],
}

pub(super) fn parse(
    archive: &[u8],
    directory_offset: usize,
    directory: &[u8],
    position: usize,
    name_budget: usize,
) -> Result<(Zip32EntrySlot, usize), Zip32Error> {
    let fixed = read::span(directory, position, 46)?;
    if &fixed[..4] != b"PK\x01\x02" {
        return Err(Zip32Error::Structure);
    }
    let name_bytes = usize::from(read::short(fixed, 28)?);
    if name_bytes > name_budget {
        return Err(Zip32Error::NameBudget);
    }
    let extra_bytes = usize::from(read::short(fixed, 30)?);
    let comment_bytes = usize::from(read::short(fixed, 32)?);
    let name_offset = read::add(position, 46)?;
    let extra_offset = read::add(name_offset, name_bytes)?;
    let comment_offset = read::add(extra_offset, extra_bytes)?;
    let end = read::add(comment_offset, comment_bytes)?;
    read::span(directory, position, end - position)?;
    let header = Header {
        version: read::short(fixed, 6)?,
        flags: read::short(fixed, 8)?,
        method: read::short(fixed, 10)?,
        crc32: read::word(fixed, 16)?,
        compressed: read::word(fixed, 20)?,
        uncompressed: read::word(fixed, 24)?,
        name: read::span(directory, name_offset, name_bytes)?,
    };
    let local_start = read::word(fixed, 42)?;
    if read::short(fixed, 34)? != 0
        || header.compressed == u32::MAX
        || header.uncompressed == u32::MAX
        || local_start == u32::MAX
    {
        return Err(Zip32Error::UnsupportedFeature);
    }
    let compression = validate(&header)?;
    extra::validate(read::span(directory, extra_offset, extra_bytes)?)?;
    let local_start = read::host(local_start)?;
    let (data_offset, local_end) =
        local::validate(&archive[..directory_offset], local_start, &header)?;
    Ok((
        Zip32EntrySlot {
            ordinal: 0,
            name_offset: read::add(directory_offset, name_offset)?,
            name_bytes,
            local_start,
            local_end,
            data_offset,
            compressed_bytes: read::host(header.compressed)?,
            uncompressed_bytes: header.uncompressed,
            crc32: header.crc32,
            compression,
        },
        end,
    ))
}

fn validate(header: &Header<'_>) -> Result<Zip32Compression, Zip32Error> {
    if !(10..=20).contains(&header.version) || header.flags & !0x080e != 0 {
        return Err(Zip32Error::UnsupportedFeature);
    }
    let compression = match header.method {
        0 if header.flags & 6 == 0 => Zip32Compression::Stored,
        8 if header.version >= 20 => Zip32Compression::Deflate,
        _ => return Err(Zip32Error::UnsupportedFeature),
    };
    if compression == Zip32Compression::Stored && header.compressed != header.uncompressed {
        return Err(Zip32Error::Structure);
    }
    if header.name.is_empty()
        || header.name.contains(&0)
        || (header.flags & 0x0800 != 0 && std::str::from_utf8(header.name).is_err())
    {
        return Err(Zip32Error::Name);
    }
    Ok(compression)
}
