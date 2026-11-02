// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent classic ZIP metadata vectors; no licensed game content is embedded.
#![allow(clippy::expect_used, reason = "Finite reviewed ZIP record fixtures.")]

#[path = "zip_cases/bounds.rs"]
mod bounds;
#[path = "zip_cases/features.rs"]
mod features;
#[path = "zip_cases/fixture.rs"]
mod fixture;
#[path = "zip_cases/layout.rs"]
mod layout;
#[path = "zip_cases/streaming.rs"]
mod streaming;
#[path = "zip_cases/views.rs"]
mod views;

use cod4xr_content::zip32::{Zip32Budget, Zip32Directory, Zip32EntrySlot, Zip32Error};
use fixture::{Spec, build, short, word};

const BUDGET: Zip32Budget = Zip32Budget {
    max_archive_bytes: 1024 * 1024,
    max_directory_bytes: 65536,
    max_entries: 1024,
    max_name_bytes: 65535,
    max_total_name_bytes: 65536,
    max_entry_uncompressed_bytes: u64::MAX,
    max_total_uncompressed_bytes: u64::MAX,
};

fn dirty_slots(count: usize) -> Vec<Zip32EntrySlot> {
    let fixture = build(&[Spec::stored(b"private/old", b"old")], &[], &[]);
    let mut slot = [Zip32EntrySlot::default()];
    Zip32Directory::parse(
        &fixture.bytes,
        BUDGET,
        &mut [Zip32EntrySlot::default()],
        &mut slot,
    )
    .expect("Seed dirty output");
    vec![slot[0]; count]
}

fn rejected(bytes: &[u8]) -> Zip32Error {
    let mut output = dirty_slots(8);
    let before = output.clone();
    let result = Zip32Directory::parse(
        bytes,
        BUDGET,
        &mut [Zip32EntrySlot::default(); 8],
        &mut output,
    )
    .expect_err("Invalid archive");
    assert_eq!(output, before);
    result
}
