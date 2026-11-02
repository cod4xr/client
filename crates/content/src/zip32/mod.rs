// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded read-only classic ZIP catalogs for IWD content, before extraction or mounting.
//! Entry names and declared checksums have no filesystem or integrity authority.

mod central;
mod directory;
mod end;
mod entry;
mod error;
mod extra;
mod local;
mod read;
mod slot;

pub use directory::Zip32Directory;
pub use entry::Zip32Entry;
pub use error::Zip32Error;
pub use slot::Zip32EntrySlot;

/// Supported entry compression identifiers, without implying verified payloads.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Zip32Compression {
    /// Method 0: bytes stored without compression.
    #[default]
    Stored,
    /// Method 8: raw Deflate, not a zlib-wrapped stream.
    Deflate,
}

/// Explicit host inspection/retention budgets, independent of native engine arenas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Zip32Budget {
    /// Maximum complete caller-provided archive bytes.
    pub max_archive_bytes: usize,
    /// Maximum encoded central-directory bytes inspected.
    pub max_directory_bytes: usize,
    /// Maximum entries inspected and retained, including directories/duplicates.
    pub max_entries: usize,
    /// Maximum raw name bytes in any one entry.
    pub max_name_bytes: usize,
    /// Maximum combined raw name bytes, counting duplicates.
    pub max_total_name_bytes: usize,
    /// Maximum declared uncompressed bytes in any one entry.
    pub max_entry_uncompressed_bytes: u64,
    /// Maximum declared uncompressed bytes across every entry.
    pub max_total_uncompressed_bytes: u64,
}

/// Metadata work/resource counts; sizes and checksums remain unverified declarations.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Zip32Stats {
    /// Complete caller-provided archive bytes.
    pub archive_bytes: usize,
    /// Encoded central-directory bytes.
    pub directory_bytes: usize,
    /// Retained entries, including directory records and duplicates.
    pub entries: usize,
    /// Raw name bytes across every entry.
    pub name_bytes: usize,
    /// Compressed payload span bytes, excluding headers/descriptors.
    pub compressed_bytes: u64,
    /// Declared uncompressed bytes, not a claim of successful decompression.
    pub declared_uncompressed_bytes: u64,
}
