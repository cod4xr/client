// SPDX-License-Identifier: AGPL-3.0-or-later
//! Checked PC IWI version-6 metadata and direct bitmap/BC mip spans, before rendering.
//! Wavelet metadata is recognized; wavelet pixel decoding is a separate codec.

mod error;
mod format;
mod header;
mod image;
mod mip;
mod read;
mod wavelet;

pub use error::Iwi6Error;
pub use format::Iwi6Format;
pub use header::{Iwi6Dimensions, Iwi6Header, Iwi6Shape};
pub use image::Iwi6DirectImage;
pub use mip::Iwi6Mip;
pub use wavelet::{Iwi6WaveletBudget, Iwi6WaveletImage, Iwi6WaveletRequirements, WaveletError};

/// Explicit host input/geometry budgets; none limit the engine's image count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Iwi6Budget {
    /// Maximum supplied encoded image bytes, checked before reading fields.
    pub max_file_bytes: usize,
    /// Maximum total logical texels across every mip, volume slice and cube face.
    pub max_texels: u64,
    /// Maximum interpreted mip levels, independently of file-byte capacity.
    pub max_mip_levels: usize,
}
