// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed image validation failures without file paths or contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pe32Error {
    /// Complete file bytes exceed the host budget.
    ImageBudget,
    /// Section count exceeds the host inspection budget.
    SectionBudget,
    /// A header or declared file extent is truncated or overflows the host.
    Truncated,
    /// DOS/PE signatures do not identify the reviewed image format.
    Signature,
    /// Machine/optional-header format is not i386 PE32.
    Format,
    /// Header/section geometry is invalid, ambiguous, unordered or outside the image.
    Geometry,
    /// The requested preferred address is not backed by file bytes.
    Address,
}
impl fmt::Display for Pe32Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ImageBudget => "PE image exceeds host byte budget",
            Self::SectionBudget => "PE section count exceeds host inspection budget",
            Self::Truncated => "PE file extent invalid or truncated",
            Self::Signature => "PE image signature invalid",
            Self::Format => "PE image format is not i386 PE32",
            Self::Geometry => "PE image header or section geometry invalid",
            Self::Address => "preferred image address has no file-backed span",
        })
    }
}
impl Error for Pe32Error {}
