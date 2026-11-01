// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt, io};

use cod4xr_client_core::{
    connect::{ConnectFailure, ConnectPlanError},
    handshake::ChallengeFailure,
};

/// Fixed network failure categories without remote payloads or request identity.
#[derive(Debug)]
pub enum NetworkError {
    /// The destination is not a nonzero-port, specified unicast endpoint.
    InvalidPeer,
    /// An OS socket operation failed.
    Io(io::Error),
    /// OS randomness for the challenge nonce was unavailable.
    Entropy(getrandom::Error),
    /// Fixed-size receive/terminal storage could not be reserved.
    Allocation,
    /// An absolute I/O deadline elapsed.
    Timeout,
    /// A monotonic deadline could not be represented.
    ClockOverflow,
    /// Deterministic challenge state rejected the operation.
    Challenge(ChallengeFailure),
    /// Deterministic connect state rejected the operation.
    Connect(ConnectFailure),
    /// Caller-supplied connect settings could not form a checked request.
    Plan(ConnectPlanError),
    /// A terminal payload exceeded the prepaid storage contract.
    ReplyCapacity,
    /// Shared ordinary-channel setup or encoding failed.
    Channel(cod4x_transport::ChannelError),
    /// This outbound message previously failed and cannot skip/retry its fragment.
    TransmissionFailed,
    /// An owned control frame belongs to another protocol/challenge context.
    ControlContext,
    /// The original connect protocol has no reviewed message profile here.
    UnsupportedProtocol,
    /// An auxiliary packet belongs to another envelope family.
    AuxiliaryKind,
    /// A checked reliable body could not be encoded or parsed.
    Reliable(cod4xr_reliable::ReliableError),
    /// Shared outer-envelope encoding failed before datagram I/O.
    Envelope(cod4x_protocol::PacketError),
    /// The original message profile does not support this reviewed auxiliary transport.
    AuxiliaryProfile,
}

impl fmt::Display for NetworkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPeer => formatter.write_str("invalid unicast peer endpoint"),
            Self::Io(error) => write!(formatter, "client network I/O failed: {error}"),
            Self::Entropy(_) => formatter.write_str("OS challenge randomness unavailable"),
            Self::Allocation => formatter.write_str("client network buffer allocation failed"),
            Self::Timeout => formatter.write_str("client network deadline elapsed"),
            Self::ClockOverflow => formatter.write_str("client network deadline overflow"),
            Self::Challenge(error) => error.fmt(formatter),
            Self::Connect(error) => error.fmt(formatter),
            Self::Plan(error) => error.fmt(formatter),
            Self::ReplyCapacity => formatter.write_str("client terminal reply capacity exceeded"),
            Self::Channel(error) => error.fmt(formatter),
            Self::TransmissionFailed => formatter.write_str("client transmission already failed"),
            Self::ControlContext => {
                formatter.write_str("client control connection context mismatch")
            }
            Self::UnsupportedProtocol => {
                formatter.write_str("unsupported original client message protocol")
            }
            Self::AuxiliaryKind => formatter.write_str("unexpected auxiliary envelope family"),
            Self::Reliable(error) => error.fmt(formatter),
            Self::Envelope(error) => error.fmt(formatter),
            Self::AuxiliaryProfile => {
                formatter.write_str("original profile does not support auxiliary reliable sending")
            }
        }
    }
}

impl Error for NetworkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Entropy(error) => Some(error),
            Self::Challenge(error) => Some(error),
            Self::Connect(error) => Some(error),
            Self::Plan(error) => Some(error),
            Self::Channel(error) => Some(error),
            Self::Reliable(error) => Some(error),
            Self::Envelope(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for NetworkError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
