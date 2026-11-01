// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::MessageError;
use std::{error::Error, fmt};

/// Fixed service-control failures without private strings or packet bodies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceControlError {
    /// A zero inspection budget cannot admit even EOF.
    InvalidBudget,
    /// The host record budget was exhausted before an admitted EOF.
    RecordBudget,
    /// Command records exceed the baseline retained window.
    CommandBudget,
    /// Final normalized command bytes exceed caller storage.
    OutputCapacity,
    /// Input ended before logical EOF.
    MissingEof,
    /// A command sequence is not positive signed 32-bit data.
    Sequence,
    /// This opcode requires another reviewed decoder; its body was not skipped.
    UnsupportedService(u8),
    /// Shared checked integer/string reading failed.
    Message(MessageError),
}

impl fmt::Display for ServiceControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidBudget => "invalid service-control inspection budget",
            Self::RecordBudget => "service-control record budget exhausted",
            Self::CommandBudget => "service-control command window exhausted",
            Self::OutputCapacity => "service-control output capacity insufficient",
            Self::MissingEof => "service-control logical EOF missing",
            Self::Sequence => "invalid service-control command sequence",
            Self::UnsupportedService(_) => "service record requires another decoder",
            Self::Message(_) => "invalid or truncated service-control record",
        })
    }
}

impl Error for ServiceControlError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Message(error) => Some(error),
            _ => None,
        }
    }
}
