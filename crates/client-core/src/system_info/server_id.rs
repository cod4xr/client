// SPDX-License-Identifier: AGPL-3.0-or-later
pub(super) fn parse(bytes: &[u8]) -> Option<u32> {
    let (negative, digits) = match bytes.first()? {
        b'-' => (true, &bytes[1..]),
        b'+' => (false, &bytes[1..]),
        _ => (false, bytes),
    };
    if digits.is_empty() {
        return None;
    }
    let magnitude = digits.iter().try_fold(0_u32, |value, &byte| {
        if !byte.is_ascii_digit() {
            return None;
        }
        value.checked_mul(10)?.checked_add(u32::from(byte - b'0'))
    })?;
    if negative {
        let signed = i32::try_from(-i64::from(magnitude)).ok()?;
        // Preserve the native signed identifier's bits, without truncating a wide value.
        Some(u32::from_ne_bytes(signed.to_ne_bytes()))
    } else {
        Some(magnitude)
    }
}
