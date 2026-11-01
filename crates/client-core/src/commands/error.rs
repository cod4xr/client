// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed command-history failures without command contents or identity data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientCommandError {
    /// Storage must contain one key slot plus 1–128 whole pending slots.
    Storage,
    /// Explicit command bytes exceed the baseline payload bound.
    CommandTooLong,
    /// Explicit command bytes contain an embedded C terminator.
    EmbeddedTerminator,
    /// All chosen pending slots still hold unacknowledged commands.
    PendingCapacity,
    /// Positive signed 32-bit sequence space is exhausted; reconnect before wrap.
    SequenceExhausted,
    /// A peer acknowledgement selected a queued but never transmitted command.
    UnsentAcknowledgement,
    /// A peer acknowledgement moved backwards from the committed position.
    AcknowledgementRegression,
    /// The selected command is outside retained history or its slot was reused.
    UnretainedSequence,
    /// A host transmit prefix moved backwards or selected an unqueued sequence.
    TransmitRange,
}

impl fmt::Display for ClientCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Storage => "invalid baseline client command storage",
            Self::CommandTooLong => "client command exceeds baseline length",
            Self::EmbeddedTerminator => "client command contains a terminator",
            Self::PendingCapacity => "client command pending capacity exhausted",
            Self::SequenceExhausted => "client command sequence requires reconnect",
            Self::UnsentAcknowledgement => {
                "client command acknowledgement exceeds transmitted work"
            }
            Self::AcknowledgementRegression => "client command acknowledgement moved backwards",
            Self::UnretainedSequence => "client command sequence is not retained",
            Self::TransmitRange => "client command transmit prefix is invalid",
        })
    }
}

impl Error for ClientCommandError {}
