// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed body codec failures without payload contents or routing values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReliableError {
    /// A complete declared header field is missing.
    Truncated,
    /// The input exceeds the selected native body profile.
    InputCapacity,
    /// A data sequence is negative/reserved or exceeds positive signed 32-bit space.
    Sequence,
    /// Cumulative acknowledgement is outside nonnegative signed 32-bit space.
    Acknowledgement,
    /// More than three selective ranges were declared.
    RangeCount,
    /// Range offset/length is not representable or length is zero.
    Range,
    /// Fragment data exceeds its native profile or has a negative declared length.
    PayloadCapacity,
    /// Declared fragment bytes differ from the complete actual body suffix.
    LengthMismatch,
    /// ACK-only sequence sentinel carried unexpected payload bytes.
    AckOnlyData,
    /// Caller cannot fit the complete encoded body.
    OutputCapacity,
}

impl fmt::Display for ReliableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Truncated => "incomplete reliable body header",
            Self::InputCapacity => "reliable body exceeds the input profile",
            Self::Sequence => "invalid reliable data sequence",
            Self::Acknowledgement => "invalid reliable cumulative acknowledgement",
            Self::RangeCount => "reliable selective range count exceeds the profile",
            Self::Range => "invalid reliable selective acknowledgement range",
            Self::PayloadCapacity => "reliable fragment exceeds the payload profile",
            Self::LengthMismatch => "reliable fragment length differs from body bytes",
            Self::AckOnlyData => "reliable ACK-only body contains data",
            Self::OutputCapacity => "reliable body output capacity insufficient",
        })
    }
}

impl Error for ReliableError {}
