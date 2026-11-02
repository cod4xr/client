// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded entropy/reconstruction codec; caller owns both work and published pixels.
mod bits;
mod entropy;
#[cfg(test)]
mod entropy_cases;
mod error;
mod image;
mod layout;
mod reconstruct;
mod tables;

use super::Iwi6Budget;
pub use error::WaveletError;
pub use image::Iwi6WaveletImage;

/// Independent encoded/geometry, decoded-byte and work-byte admission policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Iwi6WaveletBudget {
    /// Encoded input and interpreted image geometry limits.
    pub image: Iwi6Budget,
    /// Maximum decoded bytes across every mip and face.
    pub max_decoded_bytes: usize,
    /// Maximum work bytes, including a private source-mip delta copy.
    pub max_scratch_bytes: usize,
}
/// Exact caller capacities calculated before any pixel reconstruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Iwi6WaveletRequirements {
    /// Complete decoded mip/face storage, excluding untouched caller tail bytes.
    pub decoded_bytes: usize,
    /// Private reconstruction storage, including temporary source deltas.
    pub scratch_bytes: usize,
}
