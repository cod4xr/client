// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded auxiliary reliable transport, separate from ordinary channels/commands.
//! Bodies exclude the existing shared packet marker/qport envelope. No OS, game
//! runtime, allocation, authentication or application command execution is owned.

mod acks;
mod decode;
mod encode;
mod error;
mod header;
mod packet;
pub mod receive;

pub use acks::{SelectiveAckRange, SelectiveAcks};
pub use error::ReliableError;
pub use header::ReliableHeader;
pub use packet::ReliablePacket;

/// Native auxiliary fragments carry up to 1200 bytes, distinct from ordinary fragments.
pub const MAX_RELIABLE_PAYLOAD_BYTES: usize = 1200;
/// Native maximum selective acknowledgement ranges in one header.
pub const MAX_SELECTIVE_ACK_RANGES: usize = 3;
/// Body header without selective ranges, excluding the shared marker and qport.
pub const MIN_RELIABLE_HEADER_BYTES: usize = 14;
/// Largest body header, including three selective ranges.
pub const MAX_RELIABLE_HEADER_BYTES: usize =
    MIN_RELIABLE_HEADER_BYTES + 4 * MAX_SELECTIVE_ACK_RANGES;
/// Largest complete reliable body, excluding its shared six-byte outer envelope.
pub const MAX_RELIABLE_BODY_BYTES: usize = MAX_RELIABLE_HEADER_BYTES + MAX_RELIABLE_PAYLOAD_BYTES;
