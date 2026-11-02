// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent PC IWI6 header/layout vectors; synthetic pixels only.
#![allow(clippy::expect_used, reason = "Finite checked image-layout vectors.")]
#[path = "iwi_cases/bounds.rs"]
mod bounds;
#[path = "iwi_cases/geometry.rs"]
mod geometry;
#[path = "iwi_cases/header.rs"]
mod header;

use cod4xr_content::iwi6::{
    Iwi6Budget, Iwi6Dimensions, Iwi6DirectImage, Iwi6Error, Iwi6Format, Iwi6Header, Iwi6Shape,
};

const BUDGET: Iwi6Budget = Iwi6Budget {
    max_file_bytes: 1024 * 1024,
    max_texels: 1024 * 1024,
    max_mip_levels: 16,
};
fn short(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn word(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
// Payload sizes are independently specified by each vector, never computed by the decoder.
fn fixture(format: u8, flags: u8, dimensions: [u16; 3], sizes: &[usize]) -> Vec<u8> {
    let mut bytes = vec![0; 28];
    bytes[..4].copy_from_slice(b"IWi\x06");
    bytes[4] = format;
    bytes[5] = flags;
    for (index, dimension) in dimensions.into_iter().enumerate() {
        short(&mut bytes, 6 + index * 2, dimension);
    }
    for index in 0..4 {
        word(&mut bytes, 12 + index * 4, 0xdead_beef);
    }
    for (level, &size) in sizes.iter().enumerate().rev() {
        bytes.resize(
            bytes.len() + size,
            u8::try_from(level + 1).expect("Small level marker"),
        );
        if level < 4 {
            let length = u32::try_from(bytes.len()).expect("Small fixture extent");
            word(&mut bytes, 12 + level * 4, length);
        }
    }
    bytes
}
