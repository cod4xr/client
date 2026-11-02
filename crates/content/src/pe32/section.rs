// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Pe32Error, read};

#[derive(Clone, Copy)]
pub(super) struct Section {
    pub rva: u32,
    pub raw: usize,
    pub raw_size: usize,
    pub mapped_size: u32,
}

impl Section {
    pub fn parse(input: &[u8]) -> Result<Self, Pe32Error> {
        let virtual_size = read::word(input, 8)?;
        let raw_size = read::word(input, 16)?;
        Ok(Self {
            rva: read::word(input, 12)?,
            raw: read::host(read::word(input, 20)?)?,
            raw_size: read::host(raw_size)?,
            mapped_size: virtual_size.max(raw_size),
        })
    }
    pub fn end(self) -> u64 {
        u64::from(self.rva) + u64::from(self.mapped_size)
    }
}
