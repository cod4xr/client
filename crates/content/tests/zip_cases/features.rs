// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    BUDGET, Spec, Zip32Directory, Zip32EntrySlot, Zip32Error, build, rejected, short, word,
};

#[test]
fn unsupported_methods_flags_versions_and_zip64_fields_reject_explicitly() {
    for (method, flags) in [(99, 0), (9, 0), (8, 1), (8, 0x40), (8, 0x2000), (0, 2)] {
        let fixture = build(
            &[Spec {
                flags,
                method,
                ..Spec::stored(b"a", b"data")
            }],
            &[],
            &[],
        );
        assert_eq!(rejected(&fixture.bytes), Zip32Error::UnsupportedFeature);
    }
    let fixture = build(&[Spec::stored(b"a", b"data")], &[], &[]);
    for version in [0, 9, 21, 45, 0xffff] {
        let mut bytes = fixture.bytes.clone();
        short(&mut bytes, fixture.central[0] + 6, version);
        assert_eq!(rejected(&bytes), Zip32Error::UnsupportedFeature);
    }
    for field in [20, 24, 42] {
        let mut bytes = fixture.bytes.clone();
        word(&mut bytes, fixture.central[0] + field, u32::MAX);
        assert_eq!(rejected(&bytes), Zip32Error::UnsupportedFeature);
    }
    let mut bytes = fixture.bytes.clone();
    short(&mut bytes, fixture.central[0] + 34, 1);
    assert_eq!(rejected(&bytes), Zip32Error::UnsupportedFeature);
}

#[test]
fn extra_fields_have_complete_framing_and_cannot_smuggle_zip64_or_aes() {
    for extra in [&b"\x01\0\0\0"[..], b"\x01\x99\0\0"] {
        let fixture = build(
            &[Spec {
                extra,
                ..Spec::stored(b"a", b"data")
            }],
            &[],
            &[],
        );
        assert_eq!(rejected(&fixture.bytes), Zip32Error::UnsupportedFeature);
    }
    for extra in [&b"x"[..], b"xy", b"xyz", b"\xfe\xca\x03\0x"] {
        let fixture = build(
            &[Spec {
                extra,
                ..Spec::stored(b"a", b"data")
            }],
            &[],
            &[],
        );
        assert_eq!(rejected(&fixture.bytes), Zip32Error::Extra);
    }
    let fixture = build(
        &[Spec {
            extra: b"\xfe\xca\x03\0xyz",
            ..Spec::stored(b"a", b"data")
        }],
        &[],
        &[],
    );
    Zip32Directory::parse(
        &fixture.bytes,
        BUDGET,
        &mut [Zip32EntrySlot::default()],
        &mut [Zip32EntrySlot::default()],
    )
    .expect("Unknown extra fields remain inert");
    let mut bytes = fixture.bytes.clone();
    short(&mut bytes, 33, 100);
    assert_eq!(rejected(&bytes), Zip32Error::Extra);
}

#[test]
fn local_ranges_cannot_alias_even_when_duplicate_directory_records_are_well_formed() {
    let fixture = build(&[Spec::stored(b"same", b"payload")], &[0, 0], &[]);
    assert_eq!(rejected(&fixture.bytes), Zip32Error::Overlap);
}

#[test]
fn independent_python_zipfile_deflate_vector_matches_metadata_and_compressed_bytes() {
    // Python stdlib ZipFile/ZipInfo emitted this synthetic archive independently.
    let hex = concat!(
        "504b03041400000008000000215086a61036070000000500000005000000612e747874cb48cdc9c90700",
        "504b010214001400000008000000215086a610360700000005000000050000000000000000000000800100000000612e747874",
        "504b05060000000001000100330000002a0000000000",
    );
    let bytes: Vec<_> = hex
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).expect("Hex"), 16).expect("Byte"))
        .collect();
    let mut slots = [Zip32EntrySlot::default()];
    let catalog =
        Zip32Directory::parse(&bytes, BUDGET, &mut [Zip32EntrySlot::default()], &mut slots)
            .expect("Independently generated ZIP");
    let entry = catalog.entries().next().expect("One entry");
    assert_eq!(entry.name(), b"a.txt");
    assert_eq!(entry.compressed_bytes(), b"\xcb\x48\xcd\xc9\xc9\x07\0");
    assert_eq!(entry.declared_uncompressed_bytes(), 5);
    assert_eq!(entry.declared_crc32(), 0x3610_a686);
}
