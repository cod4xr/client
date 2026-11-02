// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{WaveletError, bits::Bits, entropy::Book};

pub(super) struct Level {
    pub width: usize,
    pub height: usize,
    pub channels: usize,
    pub stride: usize,
}
impl Level {
    pub fn decode(
        &self,
        bits: &mut Bits<'_>,
        source: &[u8],
        target: &mut [u8],
        delta: &mut [u8],
    ) -> Result<(), WaveletError> {
        if self.width == 1 || self.height == 1 {
            for pixel in target.chunks_exact_mut(self.stride) {
                for value in &mut pixel[..self.channels] {
                    *value = bits.raw()?;
                }
                pixel[self.channels..].fill(255);
            }
            return Ok(());
        }
        let source = if bits.take(1)? != 0 {
            let adjusted = &mut delta[..source.len()];
            adjusted.copy_from_slice(source);
            for pixel in adjusted.chunks_exact_mut(self.stride) {
                for value in &mut pixel[..self.channels] {
                    *value = wrap(i32::from(*value) + Book::Alpha.value(bits)?);
                }
            }
            adjusted
        } else {
            source
        };
        for y in (0..self.height).step_by(2) {
            for x in (0..self.width).step_by(2) {
                let parent = ((y / 2) * (self.width / 2) + x / 2) * self.stride;
                let origin = (y * self.width + x) * self.stride;
                if self.channels != 1 {
                    let parity = bits.take(1)?;
                    let blue = Book::Blue.coefficients(bits)?;
                    self.channel(target, origin, 0, source[parent], parity, blue);
                    if self.channels >= 3 {
                        for channel in 1..=2 {
                            let parity = bits.take(1)?;
                            let mut coefficients = Book::RedGreen.coefficients(bits)?;
                            for (value, reference) in coefficients.iter_mut().zip(blue) {
                                *value += reference;
                            }
                            self.channel(
                                target,
                                origin,
                                channel,
                                source[parent + channel],
                                parity,
                                coefficients,
                            );
                        }
                    }
                }
                if self.channels == 3 {
                    for offset in [
                        0,
                        self.stride,
                        self.width * self.stride,
                        (self.width + 1) * self.stride,
                    ] {
                        target[origin + offset + 3] = 255;
                    }
                } else {
                    let parity = bits.take(1)?;
                    let coefficients = Book::Alpha.coefficients(bits)?;
                    let channel = self.channels - 1;
                    self.channel(
                        target,
                        origin,
                        channel,
                        source[parent + channel],
                        parity,
                        coefficients,
                    );
                }
            }
        }
        Ok(())
    }
    fn channel(
        &self,
        pixels: &mut [u8],
        origin: usize,
        channel: usize,
        parent: u8,
        parity: u16,
        values: [i32; 3],
    ) {
        let [horizontal, vertical, diagonal] = values;
        let base = 2 * i32::from(parent);
        let row = self.width * self.stride;
        let index = origin + channel;
        pixels[index] = wrap(i32::from(parity) + ((diagonal + vertical + horizontal + base) >> 1));
        pixels[index + self.stride] = wrap((horizontal + base - diagonal - vertical) >> 1);
        pixels[index + row] = wrap((vertical - diagonal + base - horizontal) >> 1);
        pixels[index + row + self.stride] = wrap((base - horizontal - vertical + diagonal) >> 1);
    }
}
// Native reconstruction is modulo 256, including negative intermediate values.
fn wrap(value: i32) -> u8 {
    value.to_le_bytes()[0]
}
