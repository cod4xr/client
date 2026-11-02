// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent synthetic unsigned-PC fastfile/zlib vectors, without game content.
#![allow(clippy::expect_used, reason = "Finite checked container fixtures.")]
#[path = "fastfile_cases/bounds.rs"]
mod bounds;
#[path = "fastfile_cases/container.rs"]
mod container;
#[path = "fastfile_cases/lifetimes.rs"]
mod lifetimes;
use cod4xr_content::fastfile5::{
    Fastfile5Block, Fastfile5Budget, Fastfile5Decoder, Fastfile5Error,
};

const BUDGET: Fastfile5Budget = Fastfile5Budget {
    max_file_bytes: 1024 * 1024,
    max_decoded_bytes: 1024 * 1024,
    max_total_block_bytes: 1024 * 1024,
    max_external_bytes: 1024 * 1024,
};
fn inner(body: &[u8], declared: u32, external: u32, blocks: [u32; 9]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for word in [declared, external].into_iter().chain(blocks) {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes.extend_from_slice(body);
    bytes
}
fn outer(zlib: &[u8]) -> Vec<u8> {
    let mut input = b"IWffu100".to_vec();
    input.extend_from_slice(&5_u32.to_le_bytes());
    input.extend_from_slice(zlib);
    input
}
// Independent RFC1950/1951 stored-block encoder, not the production inflater.
fn stored(decoded: &[u8]) -> Vec<u8> {
    let length = u16::try_from(decoded.len()).expect("Single synthetic stored block");
    let mut zlib = vec![0x78, 0x01, 0x01];
    zlib.extend_from_slice(&length.to_le_bytes());
    zlib.extend_from_slice(&(!length).to_le_bytes());
    zlib.extend_from_slice(decoded);
    let (mut a, mut b) = (1_u32, 0_u32);
    for &byte in decoded {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    zlib.extend_from_slice(&((b << 16) | a).to_be_bytes());
    outer(&zlib)
}
// Python zlib.compress(pack('<11I',10000,777,*range(1,10))+b'abcde'*2000,9).
const HISTORY: &[u8] = &[
    0x78, 0xda, 0xed, 0xc6, 0xb9, 0x0d, 0x83, 0x00, 0x00, 0x04, 0xc1, 0x33, 0x3f, 0x99, 0xbb, 0xa0,
    0x2e, 0x1e, 0xf7, 0xdf, 0x02, 0xd7, 0x86, 0xa5, 0x09, 0x76, 0xb4, 0xdf, 0x23, 0xd9, 0xc7, 0xe4,
    0x93, 0x64, 0x68, 0xdd, 0x4c, 0x6d, 0x6e, 0x4b, 0x5b, 0xdb, 0xd6, 0xf6, 0x76, 0x5e, 0xf7, 0xf3,
    0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0xcf,
    0x79, 0x01, 0xd8, 0xfe, 0x1c, 0x82,
];
