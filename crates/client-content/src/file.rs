// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{ArchiveError, ArchiveInfo, ArchiveIoOperation};
use cod4xr_content::mount_index::ArchiveId;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

static NEXT_EPOCH: AtomicU64 = AtomicU64::new(1);
pub(crate) struct ArchiveFile {
    pub info: ArchiveInfo,
    file: File,
    modified: Option<SystemTime>,
}
impl ArchiveFile {
    pub fn open(path: &Path, maximum: u64) -> Result<Self, ArchiveError> {
        let file =
            File::open(path).map_err(|error| ArchiveError::io(ArchiveIoOperation::Open, &error))?;
        let metadata = file
            .metadata()
            .map_err(|error| ArchiveError::io(ArchiveIoOperation::Metadata, &error))?;
        if !metadata.is_file() {
            return Err(ArchiveError::NotRegular);
        }
        if metadata.len() > maximum {
            return Err(ArchiveError::FileBudget);
        }
        let id = allocate(&NEXT_EPOCH)?;
        Ok(Self {
            info: ArchiveInfo {
                id,
                bytes: metadata.len(),
            },
            file,
            modified: metadata.modified().ok(),
        })
    }
    fn check_stamp(&self) -> Result<(), ArchiveError> {
        let metadata = self
            .file
            .metadata()
            .map_err(|error| ArchiveError::io(ArchiveIoOperation::Metadata, &error))?;
        if metadata.len() != self.info.bytes || metadata.modified().ok() != self.modified {
            return Err(ArchiveError::Changed);
        }
        Ok(())
    }
    pub fn read(&mut self, offset: u64, output: &mut [u8]) -> Result<(), ArchiveError> {
        let count = u64::try_from(output.len()).map_err(|_| ArchiveError::Bounds)?;
        if offset
            .checked_add(count)
            .is_none_or(|end| end > self.info.bytes)
        {
            return Err(ArchiveError::Bounds);
        }
        self.check_stamp()?;
        self.file
            .seek(SeekFrom::Start(offset))
            .map_err(|error| ArchiveError::io(ArchiveIoOperation::Seek, &error))?;
        self.file
            .read_exact(output)
            .map_err(|error| ArchiveError::io(ArchiveIoOperation::Read, &error))?;
        self.check_stamp()
    }
}
fn allocate(counter: &AtomicU64) -> Result<ArchiveId, ArchiveError> {
    counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map(ArchiveId::new)
        .map_err(|_| ArchiveError::EpochExhausted)
}
#[cfg(test)]
mod cases {
    #![allow(clippy::expect_used, reason = "Finite private epoch-boundary test.")]
    use super::{ArchiveError, AtomicU64, allocate};
    #[test]
    fn epoch_exhaustion_rejects_without_wrapping_or_reusing_an_identifier() {
        let counter = AtomicU64::new(u64::MAX - 1);
        assert_eq!(
            allocate(&counter).expect("Final available epoch").value(),
            u64::MAX - 1
        );
        assert_eq!(allocate(&counter), Err(ArchiveError::EpochExhausted));
        assert_eq!(allocate(&counter), Err(ArchiveError::EpochExhausted));
    }
}
