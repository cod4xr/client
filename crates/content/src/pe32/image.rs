// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Pe32Budget, Pe32Error, read, section::Section};
use std::fmt;

/// Borrowed file-backed metadata view, independent of any native address space.
#[derive(Clone, Copy)]
pub struct Pe32Image<'file> {
    input: &'file [u8],
    image_base: u32,
    image_size: u32,
    headers: usize,
    section_table: &'file [u8],
    section_count: u16,
}

impl<'file> Pe32Image<'file> {
    /// Checks complete header/section extents before exposing preferred-address lookup.
    /// Section RVAs must be ordered/nonoverlapping as specified for PE images.
    /// Raw extents may contain alignment padding; virtual-only zero fill is not data.
    ///
    /// # Errors
    /// Rejects budgets, signatures, other formats, truncated/overflowing extents,
    /// directory/header geometry and unordered/overlapping/out-of-image sections.
    pub fn parse(input: &'file [u8], budget: Pe32Budget) -> Result<Self, Pe32Error> {
        if input.len() > budget.max_image_bytes {
            return Err(Pe32Error::ImageBudget);
        }
        if read::span(input, 0, 2)? != b"MZ" {
            return Err(Pe32Error::Signature);
        }
        let pe = read::host(read::word(input, 0x3c)?)?;
        if pe < 0x40 {
            return Err(Pe32Error::Geometry);
        }
        let prefix = read::span(input, pe, 24)?;
        if &prefix[..4] != b"PE\0\0" {
            return Err(Pe32Error::Signature);
        }
        if read::short(prefix, 4)? != 0x014c {
            return Err(Pe32Error::Format);
        }
        let section_count = read::short(prefix, 6)?;
        if section_count == 0 {
            return Err(Pe32Error::Geometry);
        }
        if section_count > budget.max_sections {
            return Err(Pe32Error::SectionBudget);
        }
        let optional_size = usize::from(read::short(prefix, 20)?);
        let optional_start = pe.checked_add(24).ok_or(Pe32Error::Truncated)?;
        let optional = read::span(input, optional_start, optional_size)?;
        if optional_size < 96 || read::short(optional, 0)? != 0x010b {
            return Err(Pe32Error::Format);
        }
        let directories = read::host(read::word(optional, 92)?)?;
        let directory_end = directories
            .checked_mul(8)
            .and_then(|size| size.checked_add(96))
            .ok_or(Pe32Error::Geometry)?;
        if directory_end > optional_size {
            return Err(Pe32Error::Geometry);
        }
        let image_base = read::word(optional, 28)?;
        let image_size = read::word(optional, 56)?;
        let headers = read::host(read::word(optional, 60)?)?;
        let table_start = optional_start
            .checked_add(optional_size)
            .ok_or(Pe32Error::Truncated)?;
        let table_size = usize::from(section_count)
            .checked_mul(40)
            .ok_or(Pe32Error::Truncated)?;
        let section_table = read::span(input, table_start, table_size)?;
        let table_end = table_start
            .checked_add(table_size)
            .ok_or(Pe32Error::Truncated)?;
        if headers < table_end
            || headers > input.len()
            || u64::try_from(headers).map_err(|_| Pe32Error::Geometry)? > u64::from(image_size)
            || u64::from(image_base) + u64::from(image_size) > (1_u64 << 32)
        {
            return Err(Pe32Error::Geometry);
        }
        let mut previous_end = u64::try_from(headers).map_err(|_| Pe32Error::Geometry)?;
        for bytes in section_table.chunks_exact(40) {
            let section = Section::parse(bytes)?;
            if u64::from(section.rva) < previous_end || section.end() > u64::from(image_size) {
                return Err(Pe32Error::Geometry);
            }
            if section.raw_size != 0 {
                if section.raw < headers {
                    return Err(Pe32Error::Geometry);
                }
                let _ = read::span(input, section.raw, section.raw_size)?;
            }
            previous_end = section.end();
        }
        Ok(Self {
            input,
            image_base,
            image_size,
            headers,
            section_table,
            section_count,
        })
    }

    /// Returns the reviewed preferred image base, not a process address.
    #[must_use]
    pub fn image_base(self) -> u32 {
        self.image_base
    }
    /// Returns the declared virtual image extent; no such memory is allocated.
    #[must_use]
    pub fn image_size(self) -> u32 {
        self.image_size
    }
    /// Returns inspected section headers.
    #[must_use]
    pub fn section_count(self) -> u16 {
        self.section_count
    }
    /// Borrows one complete file-backed span at a preferred virtual address.
    ///
    /// # Errors
    /// Rejects unbacked addresses, virtual zero fill and cross-section/file extents.
    pub fn file_span(self, address: u32, length: usize) -> Result<&'file [u8], Pe32Error> {
        self.file_tail(address)?
            .get(..length)
            .ok_or(Pe32Error::Address)
    }
    /// Borrows the file-backed remainder of one header/section, never virtual zero fill.
    ///
    /// # Errors
    /// Rejects addresses below the image base, gaps and virtual-only section bytes.
    pub fn file_tail(self, address: u32) -> Result<&'file [u8], Pe32Error> {
        let rva = address
            .checked_sub(self.image_base)
            .ok_or(Pe32Error::Address)?;
        let host_rva = read::host(rva)?;
        if host_rva < self.headers {
            return Ok(&self.input[host_rva..self.headers]);
        }
        for bytes in self.section_table.chunks_exact(40) {
            let section = Section::parse(bytes)?;
            if rva < section.rva || u64::from(rva) >= section.end() {
                continue;
            }
            let delta = read::host(rva - section.rva)?;
            if delta >= section.raw_size {
                return Err(Pe32Error::Address);
            }
            return Ok(&self.input[section.raw + delta..section.raw + section.raw_size]);
        }
        Err(Pe32Error::Address)
    }
}

impl fmt::Debug for Pe32Image<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Pe32Image")
            .field("file_bytes", &self.input.len())
            .field("sections", &self.section_count)
            .finish_non_exhaustive()
    }
}
