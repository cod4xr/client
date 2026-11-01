// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use super::{SendError, SendWindow};
use crate::{
    MAX_RELIABLE_PAYLOAD_BYTES, ReliableError, ReliableHeader, ReliablePacket, SelectiveAcks,
};

/// One fragment whose complete datagram was accepted by the OS, not by its peer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SentFragment {
    /// Original queue sequence, reused for retransmissions of these exact bytes.
    pub sequence: u32,
    /// Original fragment payload length, excluding any wire headers.
    pub bytes: usize,
    /// Whether this is the first successful I/O admission for this queued sequence.
    pub first_transmission: bool,
}

/// Exclusive send preparation. Drop on I/O failure; only record a successful complete send.
/// The original queued bytes/sequence remain unchanged and available for retransmission.
#[must_use = "record only successful complete datagram I/O; dropping does not mark data sent"]
pub struct PreparedTransmission<'window, 'buffers> {
    owner: &'window mut SendWindow<'buffers>,
    index: usize,
    sequence: u32,
}

impl<'buffers> SendWindow<'buffers> {
    /// Borrows one current in-flight original fragment without marking it transmitted.
    /// Successful I/O admission is separate from preparation and peer acknowledgement.
    ///
    /// # Errors
    /// Rejects missing IDs, queued data beyond the in-flight span and selectively acknowledged data.
    pub fn prepare_transmission(
        &mut self,
        sequence: u32,
    ) -> Result<PreparedTransmission<'_, 'buffers>, SendError> {
        let index = self.index(sequence)?;
        if sequence - self.base >= u32::from(self.span) {
            return Err(SendError::OutsideWindow);
        }
        if self.slots[index].selective {
            return Err(SendError::Selective);
        }
        Ok(PreparedTransmission {
            owner: self,
            index,
            sequence,
        })
    }
}

impl PreparedTransmission<'_, '_> {
    /// Returns the stable original fragment sequence.
    #[must_use]
    pub fn sequence(&self) -> u32 {
        self.sequence
    }
    /// Borrows original private payload bytes under the exclusive queue preparation.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        let start = self.index * MAX_RELIABLE_PAYLOAD_BYTES;
        &self.owner.bytes[start..start + self.owner.slots[self.index].length]
    }
    /// Builds a body view with this exact original sequence and the host's receive ACKs.
    /// The host owns receive validation, flag choice, outer qport envelope and pacing.
    ///
    /// # Errors
    /// Rejects an unrepresentable supplied cumulative ACK before any state change.
    pub fn packet(
        &self,
        acknowledge: u32,
        selective: SelectiveAcks,
        flags: u8,
    ) -> Result<ReliablePacket<'_>, ReliableError> {
        ReliablePacket::try_new(
            ReliableHeader {
                sequence: Some(self.sequence),
                acknowledge,
                flags,
                selective,
                sender_window: self.owner.hint,
            },
            self.payload(),
        )
    }
    /// Records actual successful complete datagram I/O, infallibly under the exclusive borrow.
    /// Never call this after a failed/partial OS send. It does not apply any peer ACK.
    #[must_use]
    pub fn record_sent(self) -> SentFragment {
        let slot = &mut self.owner.slots[self.index];
        let result = SentFragment {
            sequence: self.sequence,
            bytes: slot.length,
            first_transmission: !slot.transmitted,
        };
        slot.transmitted = true;
        result
    }
}

impl fmt::Debug for PreparedTransmission<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedTransmission")
            .field("sequence", &self.sequence)
            .field("payload_bytes", &self.owner.slots[self.index].length)
            .field("transmitted", &self.owner.slots[self.index].transmitted)
            .finish_non_exhaustive()
    }
}
