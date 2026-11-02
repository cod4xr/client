// SPDX-License-Identifier: AGPL-3.0-or-later
use super::InspectError;
use std::{fs::File, io::Read, path::Path};

#[cfg(test)]
#[path = "file_cases.rs"]
mod cases;

pub(super) fn read(path: &Path, maximum: usize) -> Result<Vec<u8>, InspectError> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(InspectError::NotRegular);
    }
    let capacity = usize::try_from(metadata.len()).map_err(|_| InspectError::FileBudget)?;
    if capacity > maximum {
        return Err(InspectError::FileBudget);
    }
    let limit = maximum
        .checked_add(1)
        .and_then(|value| u64::try_from(value).ok())
        .ok_or(InspectError::FileBudget)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| InspectError::Capacity)?;
    file.take(limit).read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(InspectError::FileBudget);
    }
    Ok(bytes)
}

pub(super) fn resize<T: Clone>(
    buffer: &mut Vec<T>,
    length: usize,
    fill: T,
) -> Result<(), InspectError> {
    if length > buffer.len() {
        buffer
            .try_reserve_exact(length - buffer.len())
            .map_err(|_| InspectError::Capacity)?;
    }
    buffer.resize(length, fill);
    Ok(())
}
