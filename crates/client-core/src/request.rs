// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::Packet;

use crate::{DiscoveryError, QueryChallenge};

/// Complete connectionless getinfo request length, without an unnecessary C terminator.
pub const INFO_REQUEST_BYTES: usize = 4 + 8 + 32;

impl QueryChallenge {
    /// Writes one complete getinfo request using the shared envelope codec.
    /// Failure leaves caller storage unchanged; extra output bytes are not modified.
    ///
    /// # Errors
    /// Returns [`DiscoveryError::OutputCapacity`] for insufficient output.
    pub fn write_request(&self, output: &mut [u8]) -> Result<usize, DiscoveryError> {
        if output.len() < INFO_REQUEST_BYTES {
            return Err(DiscoveryError::OutputCapacity);
        }
        let mut body = [0; INFO_REQUEST_BYTES - 4];
        body[..8].copy_from_slice(b"getinfo ");
        body[8..].copy_from_slice(self.bytes());
        Packet::Connectionless(&body)
            .encode(output)
            .map_err(|_| DiscoveryError::OutputCapacity)
    }
}
