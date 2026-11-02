// SPDX-License-Identifier: AGPL-3.0-or-later
use super::InspectError;
use cod4xr_content::defaults::ImportedDefaults;
use std::{
    fmt,
    io::{self, Write},
};

pub(super) fn write(
    image_bytes: usize,
    defaults: &ImportedDefaults<'_>,
) -> Result<(), InspectError> {
    let value = serde_json::json!({
        "project": "CoD4XR", "phase": "client_image_inspection", "image_bytes": image_bytes,
        "image_sha256": Hex(defaults.image_sha256()).to_string(),
        "default_records": defaults.record_count(), "default_value_bytes": defaults.text_bytes(),
        "maximum_default_index": defaults.maximum_index(),
        "longest_default_value": defaults.records().map(|entry| entry.bytes().len()).max().unwrap_or(0),
        "gameplay_compatibility": "unverified"
    });
    write_value(&value)
}

pub(super) fn write_value(value: &serde_json::Value) -> Result<(), InspectError> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, value).map_err(InspectError::Report)?;
    writeln!(stdout)?;
    Ok(())
}
struct Hex([u8; 32]);
impl fmt::Display for Hex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}
