// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{MAX_RELIABLE_HEADER_BYTES, ReliableError, ReliablePacket};

impl ReliablePacket<'_> {
    /// Encodes the complete body, excluding the existing shared outer envelope.
    /// Header preflight/staging finishes before any caller output changes.
    ///
    /// # Errors
    /// Rejects insufficient output or any unrepresentable field atomically.
    pub fn encode(&self, output: &mut [u8]) -> Result<usize, ReliableError> {
        let length = self.encoded_len();
        if output.len() < length {
            return Err(ReliableError::OutputCapacity);
        }
        let sequence = self
            .header
            .sequence
            .map(i32::try_from)
            .transpose()
            .map_err(|_| ReliableError::Sequence)?
            .unwrap_or(-1);
        let acknowledge =
            i32::try_from(self.header.acknowledge).map_err(|_| ReliableError::Acknowledgement)?;
        let payload_length =
            i16::try_from(self.payload.len()).map_err(|_| ReliableError::PayloadCapacity)?;
        let count = self.header.selective.len();
        let mut header = [0; MAX_RELIABLE_HEADER_BYTES];
        header[..4].copy_from_slice(&sequence.to_le_bytes());
        header[4..8].copy_from_slice(&acknowledge.to_le_bytes());
        header[8] = self.header.flags;
        header[9] = u8::try_from(count).map_err(|_| ReliableError::RangeCount)?;
        for (index, range) in self.header.selective.ranges().iter().enumerate() {
            let offset = 10 + 4 * index;
            header[offset..offset + 2].copy_from_slice(
                &i16::try_from(range.offset)
                    .map_err(|_| ReliableError::Range)?
                    .to_le_bytes(),
            );
            header[offset + 2..offset + 4].copy_from_slice(
                &i16::try_from(range.length)
                    .map_err(|_| ReliableError::Range)?
                    .to_le_bytes(),
            );
        }
        let offset = 10 + 4 * count;
        header[offset..offset + 2].copy_from_slice(&self.header.sender_window.to_le_bytes());
        header[offset + 2..offset + 4].copy_from_slice(&payload_length.to_le_bytes());
        let header_length = offset + 4;
        output[..header_length].copy_from_slice(&header[..header_length]);
        output[header_length..length].copy_from_slice(self.payload);
        Ok(length)
    }
}
