// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::Iwi6Error;
use std::{error::Error, fmt};

/// Fixed wavelet failure categories, without pixels, names or encoded bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaveletError {
    /// Explicit PC IWI6 metadata or host geometry policy failed.
    Header(Iwi6Error),
    /// A direct bitmap/block format was supplied to the wavelet codec.
    Format,
    /// Volume, non-power-of-two or missing source-mip geometry is unsupported.
    Geometry,
    /// Decoded byte extent exceeds host policy or arithmetic capacity.
    DecodedBudget,
    /// Work byte extent exceeds host policy or arithmetic capacity.
    ScratchBudget,
    /// A caller-owned output or work buffer is too short.
    Capacity,
    /// A required raw prefix byte or coded bit is missing.
    Truncated,
    /// The stream requests raw prefix bytes after coded bits began.
    RawAfterBits,
    /// An internal bit request is outside the bounded reader's range.
    BitCount,
    /// No reviewed entropy codeword matched.
    Codeword,
}
impl fmt::Display for WaveletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Header(_) => "IWI wavelet header admission failed",
            Self::Format => "IWI wavelet decoder requires a wavelet encoding",
            Self::Geometry => "IWI wavelet geometry unsupported",
            Self::DecodedBudget => "IWI wavelet decoded byte budget exhausted",
            Self::ScratchBudget => "IWI wavelet work byte budget exhausted",
            Self::Capacity => "IWI wavelet caller capacity insufficient",
            Self::Truncated => "IWI wavelet stream incomplete",
            Self::RawAfterBits => "IWI wavelet raw prefix appears after coded bits",
            Self::BitCount => "IWI wavelet bit request invalid",
            Self::Codeword => "IWI wavelet codeword invalid",
        })
    }
}
impl Error for WaveletError {}
