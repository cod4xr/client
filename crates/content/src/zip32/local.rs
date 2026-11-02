// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Zip32Error, central::Header, extra, read};

pub(super) fn validate(
    region: &[u8],
    start: usize,
    expected: &Header<'_>,
) -> Result<(usize, usize), Zip32Error> {
    let fixed = read::span(region, start, 30)?;
    if &fixed[..4] != b"PK\x03\x04" {
        return Err(Zip32Error::Structure);
    }
    if read::short(fixed, 4)? != expected.version
        || read::short(fixed, 6)? != expected.flags
        || read::short(fixed, 8)? != expected.method
    {
        return Err(Zip32Error::LocalMismatch);
    }
    let name_bytes = usize::from(read::short(fixed, 26)?);
    let extra_bytes = usize::from(read::short(fixed, 28)?);
    let name_offset = read::add(start, 30)?;
    let extra_offset = read::add(name_offset, name_bytes)?;
    let data_offset = read::add(extra_offset, extra_bytes)?;
    if read::span(region, name_offset, name_bytes)? != expected.name {
        return Err(Zip32Error::LocalMismatch);
    }
    extra::validate(read::span(region, extra_offset, extra_bytes)?)?;
    let compressed = read::host(expected.compressed)?;
    read::span(region, data_offset, compressed)?;
    let end = read::add(data_offset, compressed)?;
    let descriptor = expected.flags & 8 != 0;
    for (offset, value) in [
        (14, expected.crc32),
        (18, expected.compressed),
        (22, expected.uncompressed),
    ] {
        let actual = read::word(fixed, offset)?;
        // Stream writers may leave zero placeholders; any populated values must agree.
        if actual != value && !(descriptor && actual == 0) {
            return Err(Zip32Error::LocalMismatch);
        }
    }
    if descriptor {
        let bytes = region.get(end..).ok_or(Zip32Error::Bounds)?;
        let unsigned = descriptor_matches(bytes, expected);
        let signed = bytes
            .strip_prefix(b"PK\x07\x08")
            .is_some_and(|rest| descriptor_matches(rest, expected));
        let length = match (unsigned, signed) {
            (true, false) => 12,
            (false, true) => 16,
            _ => return Err(Zip32Error::Descriptor),
        };
        Ok((data_offset, read::add(end, length)?))
    } else {
        Ok((data_offset, end))
    }
}

fn descriptor_matches(bytes: &[u8], expected: &Header<'_>) -> bool {
    bytes.len() >= 12
        && read::word(bytes, 0) == Ok(expected.crc32)
        && read::word(bytes, 4) == Ok(expected.compressed)
        && read::word(bytes, 8) == Ok(expected.uncompressed)
}
