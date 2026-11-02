// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned original file handles, exact epoch routing and bounded CRC-checked reads.
#![allow(
    clippy::expect_used,
    reason = "Finite exclusively created temporary archive fixtures."
)]
#[path = "registry_cases/lookup.rs"]
mod lookup;
#[path = "registry_cases/mutation.rs"]
mod mutation;
#[path = "registry_cases/open.rs"]
mod open;
#[path = "registry_cases/reads.rs"]
mod reads;
#[path = "../../content/tests/zip_cases/fixture.rs"]
mod zip_fixture;

use cod4xr_client_content::{ArchiveBudget, ArchiveError, ArchiveRegistry};
use cod4xr_content::{
    zip_payload::{ZipPayloadBudget, ZipPayloadError},
    zip32::{Zip32Budget, Zip32Directory, Zip32EntryMetadata, Zip32EntrySlot},
};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const BUDGET: ArchiveBudget = ArchiveBudget {
    max_open_archives: 4,
    max_archive_bytes: 1024 * 1024,
};
const PAYLOAD: ZipPayloadBudget = ZipPayloadBudget {
    max_compressed_bytes: 1024,
    max_uncompressed_bytes: 1024,
};
const HELLO: &[u8] = b"\xcb\x48\xcd\xc9\xc9\x07\0";
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);
impl Fixture {
    fn new(bytes: &[u8]) -> Self {
        let path = std::env::temp_dir().join(format!(
            "cod4xr-PrivateArchive-{}-{}.iwd",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("Exclusive fixture");
        file.write_all(bytes).expect("Synthetic ZIP bytes");
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn image(crc: u32) -> Vec<u8> {
    zip_fixture::build(
        &[zip_fixture::Spec {
            unpacked: 5,
            crc,
            method: 8,
            ..zip_fixture::Spec::stored(b"PrivateEntry", HELLO)
        }],
        &[],
        &[],
    )
    .bytes
}
fn declarations(bytes: &[u8]) -> Vec<Zip32EntryMetadata> {
    let mut slots = [Zip32EntrySlot::default(); 8];
    let plan = Zip32Directory::parse(
        bytes,
        Zip32Budget {
            max_archive_bytes: 1024 * 1024,
            max_directory_bytes: 1024 * 1024,
            max_entries: 8,
            max_name_bytes: 1024,
            max_total_name_bytes: 8192,
            max_entry_uncompressed_bytes: 1024 * 1024,
            max_total_uncompressed_bytes: 1024 * 1024,
        },
        &mut [Zip32EntrySlot::default(); 8],
        &mut slots,
    )
    .expect("Whole catalog");
    plan.entries().map(|entry| entry.metadata()).collect()
}
