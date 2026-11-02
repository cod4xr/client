// SPDX-License-Identifier: AGPL-3.0-or-later
//! Checked file-backed preferred-address views of i386 PE32 images, not a PE loader.
//! Virtual zero-fill, relocations, imports and executable permissions are not applied.

mod error;
mod image;
mod read;
mod section;

pub use error::Pe32Error;
pub use image::Pe32Image;

/// Explicit host budgets for file bytes and inspected section headers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pe32Budget {
    /// Maximum complete caller image bytes admitted before any metadata reads.
    pub max_image_bytes: usize,
    /// Maximum section headers admitted before any section scanning.
    pub max_sections: u16,
}
