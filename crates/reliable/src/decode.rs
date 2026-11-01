// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    MAX_RELIABLE_BODY_BYTES, MAX_RELIABLE_PAYLOAD_BYTES, MAX_SELECTIVE_ACK_RANGES,
    MIN_RELIABLE_HEADER_BYTES, ReliableError, ReliableHeader, ReliablePacket, SelectiveAckRange,
    SelectiveAcks,
};

impl<'packet> ReliablePacket<'packet> {
    /// Parses a complete body after the separately checked shared marker/qport envelope.
    /// No acknowledgement is applied and no payload length controls allocation.
    ///
    /// # Errors
    /// Rejects truncation, field/profile bounds, count errors, length mismatches,
    /// negative sequence/ack data and trailing bytes instead of accepting a prefix.
    pub fn parse(bytes: &'packet [u8]) -> Result<Self, ReliableError> {
        if bytes.len() > MAX_RELIABLE_BODY_BYTES {
            return Err(ReliableError::InputCapacity);
        }
        if bytes.len() < MIN_RELIABLE_HEADER_BYTES {
            return Err(ReliableError::Truncated);
        }
        let count = usize::from(bytes[9]);
        if count > MAX_SELECTIVE_ACK_RANGES {
            return Err(ReliableError::RangeCount);
        }
        let length = MIN_RELIABLE_HEADER_BYTES + 4 * count;
        if bytes.len() < length {
            return Err(ReliableError::Truncated);
        }
        let sequence = match read_i32(bytes, 0) {
            -1 => None,
            value => Some(u32::try_from(value).map_err(|_| ReliableError::Sequence)?),
        };
        let acknowledge =
            u32::try_from(read_i32(bytes, 4)).map_err(|_| ReliableError::Acknowledgement)?;
        let mut ranges = [SelectiveAckRange::default(); MAX_SELECTIVE_ACK_RANGES];
        for (index, range) in ranges[..count].iter_mut().enumerate() {
            let offset = 10 + 4 * index;
            *range = SelectiveAckRange {
                offset: u16::try_from(read_i16(bytes, offset)).map_err(|_| ReliableError::Range)?,
                length: u16::try_from(read_i16(bytes, offset + 2))
                    .map_err(|_| ReliableError::Range)?,
            };
        }
        let offset = 10 + 4 * count;
        let payload_length = usize::try_from(read_i16(bytes, offset + 2))
            .map_err(|_| ReliableError::PayloadCapacity)?;
        if payload_length > MAX_RELIABLE_PAYLOAD_BYTES {
            return Err(ReliableError::PayloadCapacity);
        }
        if bytes.len() - length != payload_length {
            return Err(ReliableError::LengthMismatch);
        }
        Self::try_new(
            ReliableHeader {
                sequence,
                acknowledge,
                flags: bytes[8],
                selective: SelectiveAcks::new(&ranges[..count])?,
                sender_window: read_i16(bytes, offset),
            },
            &bytes[length..],
        )
    }
}

// All offsets are bounded by complete-header preflight above. This crate owns only
// this previously absent fixed body layout, not the shared outer packet codec.
fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}
fn read_i16(bytes: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}
