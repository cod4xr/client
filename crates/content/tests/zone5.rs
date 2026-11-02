// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent synthetic container-to-root-catalog vectors; no child assets.
#![allow(clippy::expect_used, reason = "Finite checked root-catalog fixtures.")]
#[path = "zone_cases/bounds.rs"]
mod bounds;
#[path = "fastfile_cases/fixture.rs"]
mod fastfile_fixture;
#[path = "zone_cases/publication.rs"]
mod publication;
#[path = "zone_cases/records.rs"]
mod records;
use cod4xr_content::{
    fastfile5::{CheckedFastfile5, Fastfile5Budget, Fastfile5Decoder},
    zone5::{
        PreparedZone5Directory, Zone5AssetKind, Zone5AssetSlot, Zone5Budget, Zone5Error,
        Zone5Output, Zone5SerializedLink, Zone5StringSlot,
    },
};
const BUDGET: Zone5Budget = Zone5Budget {
    max_script_strings: 100_000,
    max_assets: 100_000,
    max_string_bytes: 1024 * 1024,
    max_prefix_bytes: 16 * 1024 * 1024,
};
fn word(body: &mut [u8], offset: usize, value: u32) {
    body[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
fn body(strings: &[Option<&[u8]>], assets: &[(u32, u32)]) -> (Vec<u8>, u32) {
    let mut bytes = vec![0; 16];
    word(
        &mut bytes,
        0,
        u32::try_from(strings.len()).expect("String count"),
    );
    word(&mut bytes, 4, if strings.is_empty() { 0 } else { u32::MAX });
    word(
        &mut bytes,
        8,
        u32::try_from(assets.len()).expect("Asset count"),
    );
    word(&mut bytes, 12, if assets.is_empty() { 0 } else { u32::MAX });
    for string in strings {
        bytes.extend_from_slice(&if string.is_some() { u32::MAX } else { 0 }.to_le_bytes());
    }
    for string in strings.iter().flatten() {
        bytes.extend_from_slice(string);
        bytes.push(0);
    }
    let mut virtual_bytes = bytes.len() - 16;
    if !assets.is_empty() {
        virtual_bytes = virtual_bytes.next_multiple_of(4);
    }
    for &(kind, link) in assets {
        bytes.extend_from_slice(&kind.to_le_bytes());
        bytes.extend_from_slice(&link.to_le_bytes());
        virtual_bytes += 8;
    }
    (
        bytes,
        u32::try_from(virtual_bytes).expect("Synthetic virtual extent"),
    )
}
fn checked<'output>(
    body: &[u8],
    virtual_bytes: u32,
    output: &'output mut Vec<u8>,
) -> CheckedFastfile5<'output> {
    let mut blocks = [0; 9];
    blocks[4] = virtual_bytes;
    let decoded = fastfile_fixture::inner(
        body,
        u32::try_from(body.len()).expect("Fixture body"),
        0,
        blocks,
    );
    let encoded = fastfile_fixture::stored(&decoded);
    output.resize(decoded.len(), 0);
    let mut scratch = vec![0; decoded.len() + 1];
    Fastfile5Decoder::new()
        .decode(
            &encoded,
            Fastfile5Budget {
                max_file_bytes: 32 * 1024 * 1024,
                max_decoded_bytes: 32 * 1024 * 1024,
                max_total_block_bytes: u64::MAX,
                max_external_bytes: u64::MAX,
            },
            &mut scratch,
            output,
        )
        .expect("Container-checked synthetic zone")
}
