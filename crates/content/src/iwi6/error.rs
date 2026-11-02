// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed format failures without names, pixel bytes or untrusted field values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Iwi6Error {
    /// Supplied encoded bytes exceed host policy.
    FileBudget,
    /// A required header or computed payload span is missing.
    Truncated,
    /// Signature or explicit PC version-6 selection does not match.
    Profile,
    /// The stored format code is not a reviewed version-6 encoding.
    Format,
    /// Wavelet data requires its separate pixel decoder, not a direct mip view.
    Wavelet,
    /// Interpreted dimensions are zero or otherwise unrepresentable.
    Dimensions,
    /// Total interpreted texels exceed host policy or arithmetic capacity.
    TexelBudget,
    /// Interpreted mip levels exceed host policy.
    MipBudget,
    /// A computed byte extent cannot be represented on the host.
    Extent,
    /// A used picmip cumulative size differs from computed layout.
    PicmipSize,
    /// Encoded bytes remain after the complete direct mip layout.
    TrailingInput,
}
impl fmt::Display for Iwi6Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::FileBudget => "IWI file byte budget exhausted",
            Self::Truncated => "IWI header or payload span incomplete",
            Self::Profile => "IWI signature or explicit version-6 profile mismatch",
            Self::Format => "IWI encoding outside reviewed version-6 formats",
            Self::Wavelet => "IWI wavelet data requires a separate decoder",
            Self::Dimensions => "IWI interpreted dimensions invalid",
            Self::TexelBudget => "IWI logical texel budget exhausted",
            Self::MipBudget => "IWI mip-level budget exhausted",
            Self::Extent => "IWI byte extent unrepresentable on host",
            Self::PicmipSize => "IWI used picmip size disagrees with layout",
            Self::TrailingInput => "IWI bytes remain after direct mip layout",
        })
    }
}
impl Error for Iwi6Error {}
