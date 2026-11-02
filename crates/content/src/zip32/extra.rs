// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Zip32Error, read};

pub(super) fn validate(mut bytes: &[u8]) -> Result<(), Zip32Error> {
    while !bytes.is_empty() {
        if bytes.len() < 4 {
            return Err(Zip32Error::Extra);
        }
        let kind = read::short(bytes, 0)?;
        let length = usize::from(read::short(bytes, 2)?);
        if matches!(kind, 0x0001 | 0x9901) {
            return Err(Zip32Error::UnsupportedFeature);
        }
        bytes = bytes.get(4 + length..).ok_or(Zip32Error::Extra)?;
    }
    Ok(())
}
