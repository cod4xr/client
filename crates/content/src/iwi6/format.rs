// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Iwi6Dimensions, Iwi6Error};

/// Reviewed version-6 encodings, with explicit stored channel order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Iwi6Format {
    /// Four dense bytes per texel: blue, green, red, alpha.
    Bgra8,
    /// Three dense bytes per texel: blue, green, red.
    Bgr8,
    /// Two dense bytes per texel: luminance, alpha.
    LuminanceAlpha8,
    /// One luminance byte per texel.
    Luminance8,
    /// One alpha byte per texel.
    Alpha8,
    /// Wavelet-coded color/alpha; no direct block layout is implied.
    WaveletRgba,
    /// Wavelet-coded color.
    WaveletRgb,
    /// Wavelet-coded luminance/alpha.
    WaveletLuminanceAlpha,
    /// Wavelet-coded luminance.
    WaveletLuminance,
    /// Wavelet-coded alpha.
    WaveletAlpha,
    /// BC1 / DXT1, eight bytes per 4-by-4 block.
    Bc1,
    /// BC2 / DXT3, sixteen bytes per 4-by-4 block.
    Bc2,
    /// BC3 / DXT5, sixteen bytes per 4-by-4 block.
    Bc3,
    /// BC5 / DXN, sixteen bytes per 4-by-4 block.
    Bc5,
}
impl Iwi6Format {
    pub(super) fn parse(code: u8) -> Result<Self, Iwi6Error> {
        Ok(match code {
            1 => Self::Bgra8,
            2 => Self::Bgr8,
            3 => Self::LuminanceAlpha8,
            4 => Self::Luminance8,
            5 => Self::Alpha8,
            6 => Self::WaveletRgba,
            7 => Self::WaveletRgb,
            8 => Self::WaveletLuminanceAlpha,
            9 => Self::WaveletLuminance,
            10 => Self::WaveletAlpha,
            11 => Self::Bc1,
            12 => Self::Bc2,
            13 => Self::Bc3,
            14 => Self::Bc5,
            _ => return Err(Iwi6Error::Format),
        })
    }
    /// Reports whether pixel interpretation requires the separate wavelet codec.
    #[must_use]
    pub fn is_wavelet(self) -> bool {
        matches!(
            self,
            Self::WaveletRgba
                | Self::WaveletRgb
                | Self::WaveletLuminanceAlpha
                | Self::WaveletLuminance
                | Self::WaveletAlpha
        )
    }
    pub(super) fn face_bytes(self, dimensions: Iwi6Dimensions) -> Result<u64, Iwi6Error> {
        let width = u64::from(dimensions.width);
        let height = u64::from(dimensions.height);
        let depth = u64::from(dimensions.depth);
        let (columns, rows, unit) = match self {
            Self::Bgra8 => (width, height, 4),
            Self::Bgr8 => (width, height, 3),
            Self::LuminanceAlpha8 => (width, height, 2),
            Self::Luminance8 | Self::Alpha8 => (width, height, 1),
            Self::Bc1 => (width.div_ceil(4), height.div_ceil(4), 8),
            Self::Bc2 | Self::Bc3 | Self::Bc5 => (width.div_ceil(4), height.div_ceil(4), 16),
            _ => return Err(Iwi6Error::Wavelet),
        };
        columns
            .checked_mul(rows)
            .and_then(|size| size.checked_mul(depth))
            .and_then(|size| size.checked_mul(unit))
            .ok_or(Iwi6Error::Extent)
    }
}
