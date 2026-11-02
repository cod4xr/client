// SPDX-License-Identifier: AGPL-3.0-or-later
use super::InspectError;
use cod4x_observability::ConsoleFormat;
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

#[cfg(test)]
#[path = "argument_cases.rs"]
mod cases;

pub(super) const HELP: &str = "CoD4XR installed-client data inspector\n\
Usage: cod4xr-client-inspect PATH_TO_IW3MP_EXE [--log-format human|json|auto]\n\
       cod4xr-client-inspect --archive PATH_TO_IWD [--log-format human|json|auto]\n\
       cod4xr-client-inspect --verify-archive PATH_TO_IWD [--log-format human|json|auto]\n\
Imports reviewed default data and reports counts. Requires a caller-provided game file.\n\
Archive mode catalogs classic stored/Deflate ZIP metadata without extraction or CRC verification.\n\
Verification mode decodes in memory with exact length/CRC checks and no extraction to disk.\n\
Reports use JSON stdout; logs use stderr. Source values and paths stay out of logs.\n";

pub(super) struct Options {
    pub kind: Kind,
    pub path: PathBuf,
    pub logs: ConsoleFormat,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Kind {
    ClientImage,
    Archive,
    VerifiedArchive,
}
pub(super) fn from_environment() -> Result<Option<Options>, InspectError> {
    let values: Vec<_> = std::env::args_os().skip(1).take(5).collect();
    parse(values)
}
pub(super) fn parse(mut values: Vec<OsString>) -> Result<Option<Options>, InspectError> {
    if values.is_empty() {
        return Ok(None);
    }
    if values[0] == OsStr::new("--help") || values[0] == OsStr::new("-h") {
        if values.len() != 1 {
            return Err(InspectError::Arguments("unexpected help arguments"));
        }
        return Ok(None);
    }
    let kind = if values[0] == OsStr::new("--archive") {
        values.remove(0);
        Kind::Archive
    } else if values[0] == OsStr::new("--verify-archive") {
        values.remove(0);
        Kind::VerifiedArchive
    } else {
        Kind::ClientImage
    };
    if values.len() != 1 && values.len() != 3 {
        return Err(InspectError::Arguments(
            "expected a game-file path and optional log format",
        ));
    }
    if values[0].is_empty() {
        return Err(InspectError::Arguments("game-file path cannot be empty"));
    }
    let mut logs = ConsoleFormat::Auto;
    if values.len() == 3 {
        if values[1] != OsStr::new("--log-format") {
            return Err(InspectError::Arguments("unknown inspection option"));
        }
        logs = match values[2].to_str() {
            Some("human") => ConsoleFormat::Human,
            Some("json") => ConsoleFormat::Json,
            Some("auto") => ConsoleFormat::Auto,
            _ => {
                return Err(InspectError::Arguments(
                    "expected human, json or auto log format",
                ));
            }
        };
    }
    let path = values
        .into_iter()
        .next()
        .ok_or(InspectError::Arguments("game-file path missing"))?;
    Ok(Some(Options {
        kind,
        path: PathBuf::from(path),
        logs,
    }))
}
