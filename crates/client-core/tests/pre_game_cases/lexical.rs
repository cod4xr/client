// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn native_comments_interrupt_regular_tokens_and_quotes_keep_literal_comment_bytes() {
    for (bytes, map, gametype) in [
        (
            b"l/**/map/**/dm".as_slice(),
            b"map".as_slice(),
            b"dm".as_slice(),
        ),
        (b"l /*ignored*/ map // ends entire command\ndm", b"map", b""),
        (b"l map/*ignored*/dm", b"map", b"dm"),
        (
            b"l \"map//literal\" \"dm/*literal*/\"",
            b"map//literal",
            b"dm/*literal*/",
        ),
        (b"l /* ignored", b"", b""),
        (b"l map /* ignored", b"map", b""),
        (b"l // ignored\nmap dm", b"", b""),
        (b"l /*/map dm", b"map", b"dm"),
    ] {
        assert_eq!(
            inspect(bytes),
            Some(Notice::Map(map.to_vec(), gametype.to_vec()))
        );
    }
}

#[test]
fn quotes_are_separate_tokens_with_no_backslash_escapes_or_semicolon_execution() {
    for (bytes, map, gametype) in [
        (
            b"l\"map\"\"dm\"".as_slice(),
            b"map".as_slice(),
            b"dm".as_slice(),
        ),
        (b"l ma\"p\" dm", b"ma", b"p"),
        (b"l \"map\"dm", b"map", b"dm"),
        (b"l \"map with spaces", b"map with spaces", b""),
        (b"l \"map\\\"suffix dm", b"map\\", b"suffix"),
        (b"l map;quit dm", b"map;quit", b"dm"),
        (b"l \"\"dm", b"", b"dm"),
    ] {
        assert_eq!(
            inspect(bytes),
            Some(Notice::Map(map.to_vec(), gametype.to_vec()))
        );
    }
}

#[test]
fn native_control_whitespace_excludes_the_three_text_control_markers() {
    for byte in 1..=255_u8 {
        if matches!(byte, b'"' | b'/' | b'%') {
            continue;
        }
        let command = [b'l', b' ', b'a', byte, b'b', b' ', b'd', b'm'];
        let whitespace = byte <= 32 && !matches!(byte, 20..=22);
        let expected = if whitespace {
            Notice::Map(b"a".to_vec(), b"b".to_vec())
        } else {
            Notice::Map(vec![b'a', byte, b'b'], b"dm".to_vec())
        };
        assert_eq!(inspect(&command), Some(expected), "byte {byte}");
    }
}

#[test]
fn every_prefix_of_admitted_quoted_and_comment_forms_remains_bounded() {
    for source in [
        b"l/* comment */\"Private Map\"\"dm\"".as_slice(),
        b"w \"Private Reason\" /* x */ PB",
    ] {
        for end in 0..=source.len() {
            let result = inspect(&source[..end]);
            if end == 0 {
                assert_eq!(result, None);
            } else {
                assert!(result.is_some());
            }
        }
    }
}
