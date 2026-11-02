// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    BUDGET, Spec, Zip32Directory, Zip32EntrySlot, Zip32Error, build, rejected, short, word,
};

#[test]
fn terminal_geometry_counts_disks_and_zip64_sentinels_are_checked() {
    let source = build(&[Spec::stored(b"a", b"data")], &[], &[]);
    for (field, value) in [(4, 1), (6, 1), (10, u16::MAX)] {
        let mut bytes = source.bytes.clone();
        short(&mut bytes, source.end + field, value);
        assert_eq!(rejected(&bytes), Zip32Error::UnsupportedFeature);
    }
    for field in [12, 16] {
        let mut bytes = source.bytes.clone();
        word(&mut bytes, source.end + field, u32::MAX);
        assert_eq!(rejected(&bytes), Zip32Error::UnsupportedFeature);
    }
    for (field, value) in [(8, 0), (10, 0)] {
        let mut bytes = source.bytes.clone();
        short(&mut bytes, source.end + field, value);
        assert_eq!(rejected(&bytes), Zip32Error::Structure);
    }
    let mut bytes = source.bytes.clone();
    word(&mut bytes, source.end + 12, 1);
    assert_eq!(rejected(&bytes), Zip32Error::Structure);
    let mut bytes = source.bytes.clone();
    bytes[source.end] = 0;
    assert_eq!(rejected(&bytes), Zip32Error::MissingEnd);
}

#[test]
fn comments_can_contain_signatures_but_ambiguous_terminal_records_reject() {
    let source = build(
        &[Spec::stored(b"a", b"data")],
        &[],
        b"PK\x05\x06not-an-end-record",
    );
    Zip32Directory::parse(
        &source.bytes,
        BUDGET,
        &mut [Zip32EntrySlot::default()],
        &mut [Zip32EntrySlot::default()],
    )
    .expect("Inert comment signature");
    let mut nested_end = [0; 22];
    nested_end[..4].copy_from_slice(b"PK\x05\x06");
    let ambiguous = build(&[Spec::stored(b"a", b"data")], &[], &nested_end);
    assert_eq!(rejected(&ambiguous.bytes), Zip32Error::AmbiguousEnd);
    let max_comment = build(&[], &[], &vec![b'x'; usize::from(u16::MAX)]);
    Zip32Directory::parse(&max_comment.bytes, BUDGET, &mut [], &mut [])
        .expect("Full classic comment length");
}

#[test]
fn local_metadata_and_spans_must_agree_with_the_central_directory() {
    let source = build(&[Spec::stored(b"private", b"data")], &[], &[]);
    for field in [4, 6, 8, 14, 18, 22, 26] {
        let mut bytes = source.bytes.clone();
        bytes[field] ^= 1;
        assert_eq!(rejected(&bytes), Zip32Error::LocalMismatch);
    }
    let mut bytes = source.bytes.clone();
    bytes[30] ^= 1;
    assert_eq!(rejected(&bytes), Zip32Error::LocalMismatch);
    for value in [
        u32::MAX - 1,
        u32::try_from(source.central[0]).expect("Small offset"),
    ] {
        let mut bytes = source.bytes.clone();
        word(&mut bytes, source.central[0] + 42, value);
        assert_eq!(rejected(&bytes), Zip32Error::Bounds);
    }
}

#[test]
fn names_are_byte_preserving_unless_utf8_is_explicit_and_never_gain_path_authority() {
    for name in [&b"../inert"[..], b"C:/inert", b"\\inert", b"raw\x92"] {
        let fixture = build(&[Spec::stored(name, b"")], &[], &[]);
        Zip32Directory::parse(
            &fixture.bytes,
            BUDGET,
            &mut [Zip32EntrySlot::default()],
            &mut [Zip32EntrySlot::default()],
        )
        .expect("Catalog names remain inert data");
    }
    for spec in [
        Spec::stored(b"", b""),
        Spec::stored(b"bad\0name", b""),
        Spec {
            flags: 0x0800,
            ..Spec::stored(b"bad\x92", b"")
        },
    ] {
        assert_eq!(rejected(&build(&[spec], &[], &[]).bytes), Zip32Error::Name);
    }
    let unicode = build(
        &[Spec {
            flags: 0x0800,
            ..Spec::stored("目录/a".as_bytes(), b"")
        }],
        &[],
        &[],
    );
    Zip32Directory::parse(
        &unicode.bytes,
        BUDGET,
        &mut [Zip32EntrySlot::default()],
        &mut [Zip32EntrySlot::default()],
    )
    .expect("Explicit valid UTF8");
}
