// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_content::zip_payload::ZipPayloadError;
use std::{error::Error, fmt, io};

/// Safe operation context, without file paths or I/O-supplied text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveIoOperation {
    /// Opening an explicitly supplied local path.
    Open,
    /// Inspecting the original retained handle.
    Metadata,
    /// Seeking within the retained archive.
    Seek,
    /// Reading an exact bounded span.
    Read,
}
/// Fixed host failures without source paths, names or partial payload contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveError {
    /// Simultaneously open handles exceed host policy.
    OpenBudget,
    /// Source file bytes exceed host policy.
    FileBudget,
    /// Explicit source is not a regular file.
    NotRegular,
    /// Host memory reservation failed.
    Capacity,
    /// Global process handle epochs exhausted; no identifier was reused.
    EpochExhausted,
    /// An exact epoch is closed or belongs to another registry.
    Missing,
    /// Observable source length/modification stamp changed since admission.
    Changed,
    /// A copied byte extent is unrepresentable on this host or outside the admitted file.
    Bounds,
    /// Caller private compressed/image storage cannot hold the requested complete span.
    SourceCapacity,
    /// Bounded stored/raw-Deflate validation failed before final publication.
    Payload(ZipPayloadError),
    /// OS failure retaining only safe operation, kind and numeric error code.
    Io {
        /// Failed operation, without arguments.
        operation: ArchiveIoOperation,
        /// Platform-independent I/O failure class.
        kind: io::ErrorKind,
        /// Optional platform numeric code; never an OS error string or path.
        os_code: Option<i32>,
    },
}
impl ArchiveError {
    pub(crate) fn io(operation: ArchiveIoOperation, error: &io::Error) -> Self {
        Self::Io {
            operation,
            kind: error.kind(),
            os_code: error.raw_os_error(),
        }
    }
}
impl fmt::Display for ArchiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenBudget => formatter.write_str("archive open-handle budget exhausted"),
            Self::FileBudget => formatter.write_str("archive source byte budget exhausted"),
            Self::NotRegular => formatter.write_str("archive source is not a regular file"),
            Self::Capacity => formatter.write_str("archive handle reservation failed"),
            Self::EpochExhausted => formatter.write_str("archive process epochs exhausted"),
            Self::Missing => formatter.write_str("archive handle epoch not retained"),
            Self::Changed => formatter.write_str("archive source stamp changed"),
            Self::Bounds => formatter.write_str("archive byte extent outside admitted source"),
            Self::SourceCapacity => {
                formatter.write_str("archive private source capacity insufficient")
            }
            Self::Payload(error) => error.fmt(formatter),
            Self::Io {
                operation,
                kind,
                os_code,
            } => write!(
                formatter,
                "archive {operation:?} I/O failed ({kind:?}, {os_code:?})"
            ),
        }
    }
}
impl Error for ArchiveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Payload(error) => Some(error),
            _ => None,
        }
    }
}
