// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{client_data::ClientDataError, message_body::BodyReadError};
use cod4x_protocol::MessageError;
use std::{error::Error, fmt};

/// Fixed service-control failures without private strings or packet bodies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceControlError {
    /// Mixed identity controls require the explicit protocol-21 carrier.
    Profile,
    /// Identity record count exceeds its independent host budget.
    ClientDataBudget,
    /// Private identity staging cells are insufficient.
    ClientDataScratch,
    /// Final identity cells are insufficient.
    ClientDataCapacity,
    /// A complete identity update could not be decoded.
    ClientData(ClientDataError),
    /// A complete-body reader invariant failed outside shared primitive validation.
    Body(BodyReadError),
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
            Self::Profile => "mixed client-data controls require protocol 21",
            Self::ClientDataBudget => "service-control client-data budget exhausted",
            Self::ClientDataScratch => "service-control client-data scratch insufficient",
            Self::ClientDataCapacity => "service-control client-data output insufficient",
            Self::ClientData(_) => "invalid or truncated client-data service record",
            Self::Body(_) => "invalid complete service-control body view",
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
            Self::ClientData(error) => Some(error),
            Self::Body(error) => Some(error),
            Self::Message(error) => Some(error),
            _ => None,
        }
    }
}
impl From<BodyReadError> for ServiceControlError {
    fn from(value: BodyReadError) -> Self {
        match value {
            BodyReadError::Message(error) => Self::Message(error),
            other => Self::Body(other),
        }
    }
}
