// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete length-prefixed application messages over an admitted reliable stream.
//! Framing grants no command authority. No peer length controls allocation.

mod assembler;
mod error;
mod frame;

pub use assembler::{FrameAssembler, FrameProgress};
pub use error::FrameError;
pub use frame::ReliableFrame;

/// Bytes in the signed little-endian body-length prefix, excluded from that length.
pub const FRAME_PREFIX_BYTES: usize = 4;
/// Native client's full message scratch budget, including its four-byte prefix.
pub const NATIVE_MESSAGE_BYTES: usize = 1_048_576;
/// Native maximum body bytes. Explicit host budgets may differ without allocation
/// from the peer or a promise that another peer supports a larger message.
pub const NATIVE_MESSAGE_BODY_BYTES: usize = NATIVE_MESSAGE_BYTES - FRAME_PREFIX_BYTES;

/// Computes complete encoded size for a nonnegative signed-32-bit body length.
///
/// # Errors
/// Rejects unrepresentable lengths and complete-size overflow before allocation.
pub fn required_frame_bytes(body_bytes: usize) -> Result<usize, FrameError> {
    i32::try_from(body_bytes).map_err(|_| FrameError::Length)?;
    body_bytes
        .checked_add(FRAME_PREFIX_BYTES)
        .ok_or(FrameError::Length)
}

fn body_length(prefix: [u8; FRAME_PREFIX_BYTES]) -> Result<usize, FrameError> {
    usize::try_from(i32::from_le_bytes(prefix)).map_err(|_| FrameError::Length)
}
