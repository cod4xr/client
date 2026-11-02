// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed extraction failures without names, checksums or partial payload contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZipPayloadError {
    /// Compressed input exceeds the host inspection budget.
    InputBudget,
    /// Declared output exceeds the host work/output budget.
    OutputBudget,
    /// A declared output or its guard byte cannot be represented on this host.
    HostSize,
    /// Deflate scratch cannot hold declared output plus one overflow guard byte.
    ScratchCapacity,
    /// Final output cannot hold the complete declared payload.
    OutputCapacity,
    /// A complete raw Deflate stream was not decoded successfully.
    InvalidDeflate,
    /// Decoded length differs from the declaration.
    OutputLength,
    /// Bytes remain after the complete Deflate stream.
    TrailingInput,
    /// Decoded/stored bytes do not match the declared ZIP CRC32.
    Checksum,
}
impl fmt::Display for ZipPayloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InputBudget => "ZIP payload input budget exhausted",
            Self::OutputBudget => "ZIP payload output budget exhausted",
            Self::HostSize => "ZIP payload output extent unrepresentable on host",
            Self::ScratchCapacity => "ZIP payload private scratch capacity insufficient",
            Self::OutputCapacity => "ZIP payload final output capacity insufficient",
            Self::InvalidDeflate => "ZIP payload raw Deflate stream invalid",
            Self::OutputLength => "ZIP payload decoded length disagrees with declaration",
            Self::TrailingInput => "ZIP payload bytes remain after Deflate stream",
            Self::Checksum => "ZIP payload checksum disagrees with declaration",
        })
    }
}
impl Error for ZipPayloadError {}
