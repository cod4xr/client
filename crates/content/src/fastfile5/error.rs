// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed container failures, without file names, asset contents or pointer values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Fastfile5Error {
    /// Complete supplied encoded bytes exceed host policy.
    FileBudget,
    /// The outer file header is incomplete.
    Truncated,
    /// Signature or explicit unsigned PC little-endian version-5 profile differs.
    Profile,
    /// The zlib stream cannot produce the complete size header.
    SizeHeader,
    /// Decoded size exceeds host policy or cannot be represented on the host.
    DecodedBudget,
    /// Wide sum of declared memory blocks exceeds host policy.
    BlockBudget,
    /// Declared external size exceeds its independent host policy.
    ExternalBudget,
    /// Caller final output cannot hold the complete decoded section.
    OutputCapacity,
    /// Caller work space cannot hold decoded bytes plus one overflow guard.
    ScratchCapacity,
    /// Full zlib/Deflate decoding, including Adler-32 validation, failed.
    InvalidZlib,
    /// Complete decoded size disagrees with the inner size declaration.
    OutputLength,
    /// Bytes remain after the single complete zlib stream.
    TrailingInput,
}
impl fmt::Display for Fastfile5Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::FileBudget => "fastfile encoded byte budget exhausted",
            Self::Truncated => "fastfile outer header incomplete",
            Self::Profile => "fastfile unsigned PC version-5 profile mismatch",
            Self::SizeHeader => "fastfile decoded size header incomplete or invalid",
            Self::DecodedBudget => "fastfile decoded byte budget exhausted",
            Self::BlockBudget => "fastfile declared block memory budget exhausted",
            Self::ExternalBudget => "fastfile declared external size budget exhausted",
            Self::OutputCapacity => "fastfile output capacity insufficient",
            Self::ScratchCapacity => "fastfile work capacity insufficient",
            Self::InvalidZlib => "fastfile zlib integrity or complete stream validation failed",
            Self::OutputLength => "fastfile decoded length disagrees with declaration",
            Self::TrailingInput => "fastfile bytes remain after the zlib stream",
        })
    }
}
impl Error for Fastfile5Error {}
