// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent PE32 image geometry and hostile byte/address workloads.
#![allow(clippy::expect_used, reason = "Finite checked PE fixture boundaries.")]
#[path = "pe_cases/geometry.rs"]
mod geometry;
#[path = "pe_cases/views.rs"]
mod views;

use cod4xr_content::pe32::Pe32Budget;
const BUDGET: Pe32Budget = Pe32Budget {
    max_image_bytes: 0x10000,
    max_sections: 8,
};
const OPTIONAL: usize = 0x98;
const SECTIONS: usize = 0x178;

fn short(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn word(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
fn fixture() -> Vec<u8> {
    let mut bytes = vec![0; 0x600];
    bytes[..2].copy_from_slice(b"MZ");
    word(&mut bytes, 0x3c, 0x80);
    bytes[0x80..0x84].copy_from_slice(b"PE\0\0");
    short(&mut bytes, 0x84, 0x14c);
    short(&mut bytes, 0x86, 2);
    short(&mut bytes, 0x94, 224);
    short(&mut bytes, OPTIONAL, 0x10b);
    word(&mut bytes, OPTIONAL + 28, 0x0040_0000);
    word(&mut bytes, OPTIONAL + 56, 0x3000);
    word(&mut bytes, OPTIONAL + 60, 0x200);
    word(&mut bytes, OPTIONAL + 92, 16);
    for (header, virtual_size, rva, raw_size, raw) in [
        (SECTIONS, 0x180, 0x1000, 0x200, 0x200),
        (SECTIONS + 40, 0x300, 0x2000, 0x100, 0x400),
    ] {
        word(&mut bytes, header + 8, virtual_size);
        word(&mut bytes, header + 12, rva);
        word(&mut bytes, header + 16, raw_size);
        word(&mut bytes, header + 20, raw);
    }
    bytes[0x200..0x400].fill(0xa5);
    bytes[0x400..0x500].fill(0x5a);
    let marker = b"Private%Data\x92\0\0";
    bytes[0x200..0x200 + marker.len()].copy_from_slice(marker);
    bytes
}
