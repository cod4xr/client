// SPDX-License-Identifier: AGPL-3.0-or-later
//! Host-owned archive handles and bounded reads. Deterministic codecs live in content.
//! Local paths come from the host, never directly from peer configstrings or virtual keys.

mod error;
mod file;
mod open;
mod reads;

pub use error::{ArchiveError, ArchiveIoOperation};

use cod4xr_content::{mount_index::ArchiveId, zip_payload::ZipPayloadDecoder};
use std::fmt;

/// Explicit process resource policy, independent of native archive-count/file-size globals.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveBudget {
    /// Maximum simultaneously retained original file handles in this registry.
    pub max_open_archives: usize,
    /// Maximum bytes in each admitted regular archive file.
    pub max_archive_bytes: u64,
}
/// Copied identity/size of an original retained handle, without a stored source path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveInfo {
    id: ArchiveId,
    bytes: u64,
}
impl ArchiveInfo {
    /// Returns a process-unique handle epoch, never reused after closure.
    #[must_use]
    pub fn id(&self) -> ArchiveId {
        self.id
    }
    /// Returns the admitted source length, not a live filesystem query.
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
}
/// Saturating admitted-operation counters, without names, paths or payload data.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArchiveUsage {
    /// Open requests, including rejected budgets and failed I/O.
    pub open_attempts: u64,
    /// Successfully retained original handles.
    pub opened: u64,
    /// Explicitly closed retained handles.
    pub closed: u64,
    /// Whole source-image read requests, including failures.
    pub image_attempts: u64,
    /// Successfully admitted whole source-image reads.
    pub completed_images: u64,
    /// Payload read/decode requests, including preflight failures.
    pub payload_attempts: u64,
    /// Whole payloads admitted after length/stream/CRC validation.
    pub completed_payloads: u64,
    /// Source bytes in successful admitted image and payload operations.
    pub admitted_source_bytes: u64,
    /// Decoded bytes in successfully admitted payloads.
    pub admitted_payload_bytes: u64,
}

/// Exclusive owner of original regular-file handles and a reusable deterministic decoder.
/// IDs are process-unique epochs; stale/foreign registry IDs cannot alias new handles.
/// No path is retained or logged. Read results borrow caller storage only.
pub struct ArchiveRegistry {
    budget: ArchiveBudget,
    files: Vec<file::ArchiveFile>,
    decoder: ZipPayloadDecoder,
    usage: ArchiveUsage,
}
impl ArchiveRegistry {
    /// Creates an empty registry without opening files or allocating payload buffers.
    #[must_use]
    pub fn new(budget: ArchiveBudget) -> Self {
        Self {
            budget,
            files: Vec::new(),
            decoder: ZipPayloadDecoder::new(),
            usage: ArchiveUsage::default(),
        }
    }
    /// Returns resource policy without reading the filesystem.
    #[must_use]
    pub fn budget(&self) -> ArchiveBudget {
        self.budget
    }
    /// Returns operation counters; failed requests never count as admitted bytes.
    #[must_use]
    pub fn usage(&self) -> ArchiveUsage {
        self.usage
    }
    /// Returns the number of retained handles, without allocating by identifier magnitude.
    #[must_use]
    pub fn open_count(&self) -> usize {
        self.files.len()
    }
    /// Returns copied identity/size for an exact live epoch, without I/O.
    #[must_use]
    pub fn info(&self, id: ArchiveId) -> Option<ArchiveInfo> {
        self.files
            .iter()
            .find(|file| file.info.id == id)
            .map(|file| file.info)
    }
    /// Closes one exact epoch. It will never route to a later reopened handle.
    pub fn close(&mut self, id: ArchiveId) -> bool {
        let Some(position) = self.files.iter().position(|file| file.info.id == id) else {
            return false;
        };
        self.files.swap_remove(position);
        self.usage.closed = self.usage.closed.saturating_add(1);
        true
    }
}
impl fmt::Debug for ArchiveRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ArchiveRegistry")
            .field("open_archives", &self.files.len())
            .field("budget", &self.budget)
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
