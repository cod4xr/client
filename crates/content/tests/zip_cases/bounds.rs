// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    BUDGET, Spec, Zip32Budget, Zip32Directory, Zip32EntrySlot, Zip32Error, build, dirty_slots,
    rejected,
};

#[test]
fn every_truncated_fixture_preserves_final_output_and_extra_bytes_reject() {
    let source = build(
        &[
            Spec::stored(b"one", b"private-payload"),
            Spec::stored(b"two", b"last"),
        ],
        &[],
        &[],
    );
    for end in 0..source.bytes.len() {
        let _ = rejected(&source.bytes[..end]);
    }
    for byte in 0..=255 {
        let mut bytes = source.bytes.clone();
        bytes.push(byte);
        let _ = rejected(&bytes);
    }
}

#[test]
fn all_host_budgets_and_caller_capacities_are_exact_and_atomic() {
    let fixture = build(
        &[Spec::stored(b"one", b"123"), Spec::stored(b"two2", b"4567")],
        &[],
        &[],
    );
    let exact = Zip32Budget {
        max_archive_bytes: fixture.bytes.len(),
        max_directory_bytes: fixture.end - fixture.central[0],
        max_entries: 2,
        max_name_bytes: 4,
        max_total_name_bytes: 7,
        max_entry_uncompressed_bytes: 4,
        max_total_uncompressed_bytes: 7,
    };
    let cases = [
        (
            Zip32Budget {
                max_archive_bytes: exact.max_archive_bytes - 1,
                ..exact
            },
            Zip32Error::ArchiveBudget,
        ),
        (
            Zip32Budget {
                max_directory_bytes: exact.max_directory_bytes - 1,
                ..exact
            },
            Zip32Error::DirectoryBudget,
        ),
        (
            Zip32Budget {
                max_entries: 1,
                ..exact
            },
            Zip32Error::EntryBudget,
        ),
        (
            Zip32Budget {
                max_name_bytes: 3,
                ..exact
            },
            Zip32Error::NameBudget,
        ),
        (
            Zip32Budget {
                max_total_name_bytes: 6,
                ..exact
            },
            Zip32Error::NameBudget,
        ),
        (
            Zip32Budget {
                max_entry_uncompressed_bytes: 3,
                ..exact
            },
            Zip32Error::UncompressedBudget,
        ),
        (
            Zip32Budget {
                max_total_uncompressed_bytes: 6,
                ..exact
            },
            Zip32Error::UncompressedBudget,
        ),
    ];
    for (budget, expected) in cases {
        let mut output = dirty_slots(2);
        let before = output.clone();
        assert_eq!(
            Zip32Directory::parse(
                &fixture.bytes,
                budget,
                &mut [Zip32EntrySlot::default(); 2],
                &mut output
            )
            .err(),
            Some(expected)
        );
        assert_eq!(output, before);
    }
    Zip32Directory::parse(
        &fixture.bytes,
        exact,
        &mut [Zip32EntrySlot::default(); 2],
        &mut [Zip32EntrySlot::default(); 2],
    )
    .expect("Exact bounds");
}

#[test]
fn caller_capacities_are_checked_before_final_output_changes() {
    let fixture = build(
        &[Spec::stored(b"one", b"123"), Spec::stored(b"two2", b"4567")],
        &[],
        &[],
    );
    for (scratch_count, output_count, expected) in [
        (1, 2, Zip32Error::ScratchCapacity),
        (2, 1, Zip32Error::OutputCapacity),
    ] {
        let mut output = dirty_slots(output_count);
        let before = output.clone();
        assert_eq!(
            Zip32Directory::parse(
                &fixture.bytes,
                BUDGET,
                &mut vec![Zip32EntrySlot::default(); scratch_count],
                &mut output
            )
            .err(),
            Some(expected)
        );
        assert_eq!(output, before);
    }
}

#[test]
fn deterministic_record_mutations_never_publish_on_failure_or_expose_unbounded_spans() {
    let fixture = build(&[Spec::stored(b"synthetic", b"payload")], &[], &[]);
    for offset in 0..fixture.bytes.len() {
        for mask in [1, 0x80, 0xff] {
            let mut bytes = fixture.bytes.clone();
            bytes[offset] ^= mask;
            let mut output = dirty_slots(2);
            let before = output.clone();
            match Zip32Directory::parse(
                &bytes,
                BUDGET,
                &mut [Zip32EntrySlot::default(); 2],
                &mut output,
            ) {
                Ok(catalog) => {
                    assert_eq!(catalog.entries().len(), catalog.stats().entries);
                    for entry in catalog.entries() {
                        assert!(entry.name().len() <= bytes.len());
                        assert!(entry.compressed_bytes().len() <= bytes.len());
                    }
                }
                Err(_) => assert_eq!(output, before),
            }
        }
    }
}
