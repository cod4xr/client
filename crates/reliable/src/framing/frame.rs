// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use super::{FRAME_PREFIX_BYTES, FrameError, body_length, required_frame_bytes};

/// Complete application body with a representable length, still untrusted commands.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ReliableFrame<'body> {
    pub(super) body: &'body [u8],
}

impl<'body> ReliableFrame<'body> {
    /// Borrows a body without allocating or granting execution authority.
    /// A zero-length body is structurally representable; application validation is separate.
    ///
    /// # Errors
    /// Rejects a body length outside signed-32-bit/host encoded-size space.
    pub fn new(body: &'body [u8]) -> Result<Self, FrameError> {
        required_frame_bytes(body.len())?;
        Ok(Self { body })
    }

    /// Parses one exact complete frame, rejecting both missing and trailing body bytes.
    ///
    /// # Errors
    /// Rejects incomplete/negative/unrepresentable lengths and inexact body suffixes.
    pub fn parse(bytes: &'body [u8]) -> Result<Self, FrameError> {
        let prefix: [u8; FRAME_PREFIX_BYTES] = bytes
            .get(..FRAME_PREFIX_BYTES)
            .ok_or(FrameError::Truncated)?
            .try_into()
            .map_err(|_| FrameError::Truncated)?;
        let length = body_length(prefix)?;
        if required_frame_bytes(length)? != bytes.len() {
            return Err(FrameError::LengthMismatch);
        }
        Self::new(&bytes[FRAME_PREFIX_BYTES..])
    }

    /// Returns untrusted body bytes. Typed command decoding must validate the entire body.
    #[must_use]
    pub fn body(&self) -> &'body [u8] {
        self.body
    }

    /// Returns complete encoded bytes including the four-byte prefix.
    #[must_use]
    pub fn encoded_len(&self) -> usize {
        FRAME_PREFIX_BYTES + self.body.len()
    }

    /// Returns the validated four-byte body length for scatter/gather queue ownership.
    ///
    /// # Errors
    /// Rejects an unrepresentable body length without changing any output.
    pub fn length_prefix(&self) -> Result<[u8; FRAME_PREFIX_BYTES], FrameError> {
        Ok(i32::try_from(self.body.len())
            .map_err(|_| FrameError::Length)?
            .to_le_bytes())
    }

    /// Encodes a complete prefix and body after preflighting all output bounds.
    ///
    /// # Errors
    /// Rejects insufficient output or unrepresentable lengths without changing output.
    pub fn encode(&self, output: &mut [u8]) -> Result<usize, FrameError> {
        let length = self.encoded_len();
        if output.len() < length {
            return Err(FrameError::OutputCapacity);
        }
        let prefix = self.length_prefix()?;
        output[..FRAME_PREFIX_BYTES].copy_from_slice(&prefix);
        output[FRAME_PREFIX_BYTES..length].copy_from_slice(self.body);
        Ok(length)
    }
}

impl fmt::Debug for ReliableFrame<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReliableFrame")
            .field("body_bytes", &self.body.len())
            .finish_non_exhaustive()
    }
}
