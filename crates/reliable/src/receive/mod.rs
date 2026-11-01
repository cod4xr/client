// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller-funded receive windows. Fragment preparation never admits peer ACKs.
//! Hosts must validate the entire packet and outgoing ACK update before committing
//! a prepared incoming fragment. No timers, automatic replies or peer-selected budgets.

mod error;
mod prepared;
mod selective;
mod stream;
#[cfg(test)]
mod tests;
mod window;

pub use error::ReceiveError;
pub use prepared::{PreparedFragment, ReceiveDisposition};
pub use selective::ReceiveAcknowledgement;
pub use stream::ReadProgress;
pub use window::{ReceiveSlot, ReceiveWindow};

use crate::MAX_RELIABLE_PAYLOAD_BYTES;

/// Native default receive span, independent of the ordinary channel fragment size.
pub const NATIVE_RECEIVE_WINDOW_FRAGMENTS: usize = 20;
/// Largest explicit span representable by the native selective signed-short fields.
pub const MAX_RECEIVE_WINDOW_FRAGMENTS: usize = 32767;

/// Computes exact caller byte storage for an explicitly selected fragment span.
/// This is not permission to increase another peer's negotiated wire limits.
///
/// # Errors
/// Rejects empty/overlarge spans and multiplication overflow before allocating.
pub fn required_receive_bytes(fragments: usize) -> Result<usize, ReceiveError> {
    if fragments == 0 || fragments > MAX_RECEIVE_WINDOW_FRAGMENTS {
        return Err(ReceiveError::Storage);
    }
    fragments
        .checked_mul(MAX_RELIABLE_PAYLOAD_BYTES)
        .ok_or(ReceiveError::Storage)
}
