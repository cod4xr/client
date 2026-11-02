// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{ArchiveError, ArchiveInfo, ArchiveRegistry, file::ArchiveFile};
use std::path::Path;

impl ArchiveRegistry {
    /// Opens an explicitly host-approved local regular archive and retains its original handle.
    /// Never pass peer strings or virtual keys as host paths. IDs are process-local epochs,
    /// not persistent cache identities. Failed opens retain no new file or path.
    ///
    /// # Errors
    /// Rejects handle/file budgets, memory reservation, nonregular files, epoch exhaustion
    /// and OS failures. Diagnostic values never retain or echo the supplied path.
    pub fn open_local_file(&mut self, path: &Path) -> Result<ArchiveInfo, ArchiveError> {
        self.usage.open_attempts = self.usage.open_attempts.saturating_add(1);
        if self.files.len() >= self.budget.max_open_archives {
            return Err(ArchiveError::OpenBudget);
        }
        self.files
            .try_reserve(1)
            .map_err(|_| ArchiveError::Capacity)?;
        let file = ArchiveFile::open(path, self.budget.max_archive_bytes)?;
        let info = file.info;
        self.files.push(file);
        self.usage.opened = self.usage.opened.saturating_add(1);
        Ok(info)
    }
}
