// SPDX-License-Identifier: AGPL-3.0-or-later
//! Synthetic wavelet vectors independent of proprietary game data.
#![allow(clippy::expect_used, reason = "Finite synthetic wavelet vectors.")]
#[path = "wavelet_cases/bounds.rs"]
mod bounds;
#[path = "wavelet_cases/chains.rs"]
mod chains;
#[path = "wavelet_cases/pixels.rs"]
mod pixels;
use cod4xr_content::iwi6::{
    Iwi6Budget, Iwi6Format, Iwi6WaveletBudget, Iwi6WaveletImage, WaveletError,
};

const BUDGET: Iwi6WaveletBudget = Iwi6WaveletBudget {
    image: Iwi6Budget {
        max_file_bytes: 1024 * 1024,
        max_texels: 1024 * 1024,
        max_mip_levels: 16,
    },
    max_decoded_bytes: 4 * 1024 * 1024,
    max_scratch_bytes: 8 * 1024 * 1024,
};
fn fixture(format: u8, flags: u8, width: u16, height: u16, payload: &[u8]) -> Vec<u8> {
    let mut input = vec![0; 28];
    input[..4].copy_from_slice(b"IWi\x06");
    input[4] = format;
    input[5] = flags;
    input[6..8].copy_from_slice(&width.to_le_bytes());
    input[8..10].copy_from_slice(&height.to_le_bytes());
    input[10..12].copy_from_slice(&1_u16.to_le_bytes());
    input.extend_from_slice(payload);
    input
}
#[derive(Default)]
struct Stream {
    bytes: Vec<u8>,
    bits: usize,
}
impl Stream {
    fn raw(input: &[u8]) -> Self {
        Self {
            bytes: input.to_vec(),
            bits: input.len() * 8,
        }
    }
    fn put(&mut self, value: u16, count: usize) {
        for shift in 0..count {
            if self.bits / 8 == self.bytes.len() {
                self.bytes.push(0);
            }
            self.bytes[self.bits / 8] |=
                u8::try_from((value >> shift) & 1).expect("one bit") << (self.bits % 8);
            self.bits += 1;
        }
    }
    fn alpha(&mut self, value: i16) {
        if value == 0 {
            self.put(1, 1);
        } else {
            self.put(0, 4);
            self.put(u16::try_from(value + 255).expect("9-bit alpha escape"), 9);
        }
    }
    fn triplet(&mut self, channels: usize) {
        if channels != 1 {
            self.put(0, 1); // blue parity
            for _ in 0..3 {
                self.put(1, 3);
            } // blue zero
            if channels >= 3 {
                for _ in 0..2 {
                    self.put(0, 1);
                    for _ in 0..3 {
                        self.put(3, 2);
                    } // green/red zero
                }
            }
        }
        if channels != 3 {
            self.put(0, 1);
            for _ in 0..3 {
                self.alpha(0);
            }
        }
    }
}
