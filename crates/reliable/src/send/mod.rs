// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller-funded outgoing fragments with explicit I/O admission and whole ACK preflight.
//! Timing, pacing, retransmission scheduling and OS sends remain outside this crate.

mod acknowledgement;
mod error;
mod queue;
#[cfg(test)]
mod tests;
mod transmission;
mod window;

pub use acknowledgement::{AcknowledgementUpdate, PreparedAcknowledgement};
pub use error::SendError;
pub use queue::QueuedFragments;
pub use transmission::{PreparedTransmission, SentFragment};
pub use window::{SendSlot, SendWindow};

use crate::MAX_RELIABLE_PAYLOAD_BYTES;

/// Native default number of outstanding fragments eligible for transmission.
pub const NATIVE_SEND_WINDOW_FRAGMENTS: u16 = 4;
/// Largest explicitly funded span representable by the native signed window hint.
pub const MAX_SEND_WINDOW_FRAGMENTS: u16 = 32767;

/// Computes exact caller payload storage without imposing the native dynamic-buffer size.
///
/// # Errors
/// Rejects empty metadata and size multiplication overflow before allocating.
pub fn required_send_bytes(fragments: usize) -> Result<usize, SendError> {
    if fragments == 0 {
        return Err(SendError::Storage);
    }
    fragments
        .checked_mul(MAX_RELIABLE_PAYLOAD_BYTES)
        .ok_or(SendError::Storage)
}
