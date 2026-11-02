// SPDX-License-Identifier: AGPL-3.0-or-later
// Only the first three native arguments affect these handlers. Subsequent tokens
// cannot change their decisions, so no global token/text pool is reproduced.
pub(super) fn first_three(bytes: &[u8]) -> [Option<&[u8]>; 3] {
    let mut remaining = bytes;
    std::array::from_fn(|_| next(&mut remaining))
}

fn whitespace(byte: u8) -> bool {
    matches!(byte, 1..=32) && !matches!(byte, 20..=22)
}
fn next<'source>(remaining: &mut &'source [u8]) -> Option<&'source [u8]> {
    loop {
        let skipped = remaining
            .iter()
            .take_while(|&&byte| whitespace(byte))
            .count();
        *remaining = &remaining[skipped..];
        if remaining.is_empty() || remaining.starts_with(b"//") {
            *remaining = &[];
            return None;
        }
        if !remaining.starts_with(b"/*") {
            break;
        }
        let Some(end) = remaining.windows(2).position(|pair| pair == b"*/") else {
            *remaining = &[];
            return None;
        };
        *remaining = &remaining[end + 2..];
    }
    if remaining[0] == b'"' {
        let tail = &remaining[1..];
        let end = tail.iter().position(|&byte| byte == b'"');
        *remaining = end.map_or(&[], |end| &tail[end + 1..]);
        return Some(&tail[..end.unwrap_or(tail.len())]);
    }
    let end = remaining
        .iter()
        .enumerate()
        .position(|(index, &byte)| {
            whitespace(byte)
                || byte == b'"'
                || remaining[index..].starts_with(b"//")
                || remaining[index..].starts_with(b"/*")
        })
        .unwrap_or(remaining.len());
    let token = &remaining[..end];
    *remaining = &remaining[end..];
    Some(token)
}
