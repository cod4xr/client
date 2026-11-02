// SPDX-License-Identifier: AGPL-3.0-or-later
//! Whole-stream extraction contracts over independent synthetic ZIP/Deflate vectors.
#![allow(clippy::expect_used, reason = "Finite reviewed extraction fixtures.")]

#[path = "payload_cases/bounds.rs"]
mod bounds;
#[path = "payload_cases/dynamic.rs"]
mod dynamic;
#[path = "zip_cases/fixture.rs"]
mod fixture;
#[path = "payload_cases/metadata.rs"]
mod metadata;
#[path = "payload_cases/streams.rs"]
mod streams;
#[path = "payload_cases/views.rs"]
mod views;

use cod4xr_content::{
    zip_payload::{ZipPayloadBudget, ZipPayloadDecoder, ZipPayloadError},
    zip32::{Zip32Budget, Zip32Directory, Zip32Entry, Zip32EntrySlot},
};
use fixture::{Fixture, Spec, build, short, word};

const BUDGET: ZipPayloadBudget = ZipPayloadBudget {
    max_compressed_bytes: 1024 * 1024,
    max_uncompressed_bytes: 1024 * 1024,
};
const HELLO: &[u8] = b"\xcb\x48\xcd\xc9\xc9\x07\0";
const HELLO_CRC: u32 = 0x3610_a686;

fn source(packed: &[u8], unpacked: u32, crc: u32, method: u16) -> Fixture {
    build(
        &[Spec {
            unpacked,
            crc,
            method,
            ..Spec::stored(b"PrivateEntry", packed)
        }],
        &[],
        &[],
    )
}
fn entry(bytes: &[u8]) -> Zip32Entry<'_> {
    let mut slots = [Zip32EntrySlot::default()];
    let catalog = Zip32Directory::parse(
        bytes,
        Zip32Budget {
            max_archive_bytes: usize::MAX,
            max_directory_bytes: usize::MAX,
            max_entries: 1,
            max_name_bytes: usize::MAX,
            max_total_name_bytes: usize::MAX,
            max_entry_uncompressed_bytes: u64::MAX,
            max_total_uncompressed_bytes: u64::MAX,
        },
        &mut [Zip32EntrySlot::default()],
        &mut slots,
    )
    .expect("Structurally valid catalog");
    catalog.entries().next().expect("One entry")
}
fn hex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).expect("Hex"), 16).expect("Byte"))
        .collect()
}
