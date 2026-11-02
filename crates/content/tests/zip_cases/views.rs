// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, Spec, Zip32Directory, Zip32EntrySlot, build, dirty_slots};
use cod4xr_content::zip32::Zip32Compression;

#[test]
fn complete_catalog_retains_raw_bytes_order_and_only_declared_integrity() {
    let deflate = Spec {
        method: 8,
        flags: 2,
        unpacked: 1_000_000,
        ..Spec::stored(b"private%\x92", b"not-a-verified-stream")
    };
    let source = build(
        &[Spec::stored(b"folder/", b""), deflate],
        &[1, 0],
        b"private-comment",
    );
    let mut slots = dirty_slots(4);
    let tail = slots[2..].to_vec();
    let mut scratch = dirty_slots(5);
    let scratch_tail = scratch[2..].to_vec();
    let directory = Zip32Directory::parse(&source.bytes, BUDGET, &mut scratch, &mut slots)
        .expect("Whole metadata catalog");
    assert_eq!(&scratch[2..], &scratch_tail);
    scratch.fill(Zip32EntrySlot::default());
    let entries: Vec<_> = directory.entries().collect();
    assert_eq!(entries[0].name(), b"private%\x92");
    assert_eq!(entries[0].compression(), Zip32Compression::Deflate);
    assert_eq!(entries[0].compressed_bytes(), b"not-a-verified-stream");
    assert_eq!(entries[0].declared_uncompressed_bytes(), 1_000_000);
    assert_eq!(entries[0].declared_crc32(), 0x1234_5678);
    assert_eq!(entries[0].ordinal(), 0);
    assert!(!entries[0].has_directory_marker());
    assert!(entries[1].has_directory_marker());
    assert_eq!(entries[1].ordinal(), 1);
    assert_eq!(directory.stats().entries, 2);
    assert_eq!(directory.stats().name_bytes, 16);
    assert_eq!(directory.stats().declared_uncompressed_bytes, 1_000_000);
    let debug = format!("{directory:?} {:?} {:?}", entries[0], entries[1]);
    for secret in ["private", "folder", "verified", "comment", "12345678"] {
        assert!(!debug.contains(secret));
    }
    assert_eq!(&slots[2..], &tail);
}

#[test]
fn copied_entry_views_outlive_parent_catalog_and_caller_descriptors() {
    let fixture = build(&[Spec::stored(b"a", b"data")], &[], &[]);
    let entry = {
        let mut slots = [Zip32EntrySlot::default()];
        let catalog = Zip32Directory::parse(
            &fixture.bytes,
            BUDGET,
            &mut [Zip32EntrySlot::default()],
            &mut slots,
        )
        .expect("Catalog");
        catalog.entries().next().expect("One entry")
    };
    assert_eq!(entry.name(), b"a");
    assert_eq!(entry.compressed_bytes(), b"data");
}

#[test]
fn repeated_names_remain_distinct_and_empty_archives_need_no_storage() {
    let fixture = build(
        &[
            Spec::stored(b"same", b"first"),
            Spec::stored(b"same", b"second"),
        ],
        &[],
        &[],
    );
    let mut slots = [Zip32EntrySlot::default(); 2];
    let directory = Zip32Directory::parse(
        &fixture.bytes,
        BUDGET,
        &mut [Zip32EntrySlot::default(); 2],
        &mut slots,
    )
    .expect("Duplicate names are catalog data");
    assert_eq!(
        directory
            .entries()
            .map(|entry| entry.compressed_bytes())
            .collect::<Vec<_>>(),
        [&b"first"[..], b"second"]
    );
    let empty = build(&[], &[], &[]);
    let catalog = Zip32Directory::parse(&empty.bytes, BUDGET, &mut [], &mut []).expect("Empty ZIP");
    assert_eq!(catalog.entries().len(), 0);
    assert_eq!(catalog.stats().declared_uncompressed_bytes, 0);
}
