// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_content::defaults::DefaultsError;
use cod4xr_content::zip_payload::ZipPayloadError;
use cod4xr_content::zip32::Zip32Error;
use std::{error::Error, fmt, io};

#[derive(Debug)]
pub(crate) enum InspectError {
    Arguments(&'static str),
    Logging(cod4x_observability::LogError),
    Io(io::Error),
    NotRegular,
    FileBudget,
    Import(DefaultsError),
    Archive(Zip32Error),
    Payload(ZipPayloadError),
    Capacity,
    Report(serde_json::Error),
}
impl fmt::Display for InspectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arguments(message) => formatter.write_str(message),
            Self::Logging(error) => write!(formatter, "inspection logging setup failed: {error}"),
            Self::Io(error) => write!(formatter, "inspection file/output I/O failed: {error}"),
            Self::NotRegular => formatter.write_str("inspection requires a regular game file"),
            Self::FileBudget => formatter.write_str("inspection file byte budget exceeded"),
            Self::Import(error) => error.fmt(formatter),
            Self::Archive(error) => error.fmt(formatter),
            Self::Payload(error) => error.fmt(formatter),
            Self::Capacity => formatter.write_str("inspection memory reservation failed"),
            Self::Report(error) => write!(formatter, "inspection report failed: {error}"),
        }
    }
}
impl Error for InspectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Logging(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Import(error) => Some(error),
            Self::Archive(error) => Some(error),
            Self::Payload(error) => Some(error),
            Self::Report(error) => Some(error),
            _ => None,
        }
    }
}
impl From<io::Error> for InspectError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
