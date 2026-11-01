// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{MAX_RELIABLE_PAYLOAD_BYTES, MIN_RELIABLE_HEADER_BYTES, ReliableError, ReliableHeader};
use std::fmt;

/// Borrowed complete reliable body. Parsing alone admits no ACK, window or command.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ReliablePacket<'packet> {
    pub(super) header: ReliableHeader,
    pub(super) payload: &'packet [u8],
}

impl<'packet> ReliablePacket<'packet> {
    /// Admits representable metadata and bounded payload without allocating or sending.
    /// A zero-length data fragment is representable; ACK-only packets must be empty.
    ///
    /// # Errors
    /// Rejects signed sequence/ack domains, oversized payload and ACK-only data.
    pub fn try_new(header: ReliableHeader, payload: &'packet [u8]) -> Result<Self, ReliableError> {
        if header
            .sequence
            .is_some_and(|value| i32::try_from(value).is_err())
        {
            return Err(ReliableError::Sequence);
        }
        if i32::try_from(header.acknowledge).is_err() {
            return Err(ReliableError::Acknowledgement);
        }
        if payload.len() > MAX_RELIABLE_PAYLOAD_BYTES {
            return Err(ReliableError::PayloadCapacity);
        }
        if header.sequence.is_none() && !payload.is_empty() {
            return Err(ReliableError::AckOnlyData);
        }
        Ok(Self { header, payload })
    }
    /// Returns copied metadata without advancing any stateful window.
    #[must_use]
    pub fn header(&self) -> ReliableHeader {
        self.header
    }
    /// Borrows untrusted fragment bytes. Never execute or log their contents.
    #[must_use]
    pub fn payload(&self) -> &'packet [u8] {
        self.payload
    }
    /// Returns complete body bytes, excluding the shared packet marker/qport envelope.
    #[must_use]
    pub fn encoded_len(&self) -> usize {
        MIN_RELIABLE_HEADER_BYTES + 4 * self.header.selective.len() + self.payload.len()
    }
}

impl fmt::Debug for ReliablePacket<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReliablePacket")
            .field("header", &self.header)
            .field("payload_bytes", &self.payload.len())
            .finish_non_exhaustive()
    }
}
