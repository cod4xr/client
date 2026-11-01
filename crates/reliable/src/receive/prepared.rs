// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use super::{ReceiveError, ReceiveSlot, ReceiveWindow};
use crate::{MAX_RELIABLE_PAYLOAD_BYTES, ReliablePacket};

/// Incoming fragment disposition after receive-only preflight.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiveDisposition {
    /// No data sequence; this grants no authority to the packet's outgoing ACKs.
    AckOnly,
    /// First accepted bytes for a sequence in the current window.
    Stored {
        /// Exact original fragment sequence.
        sequence: u32,
        /// Retained payload byte count, possibly zero.
        bytes: usize,
    },
    /// This sequence is already retained; conflicting retransmission bytes are ignored.
    Duplicate {
        /// Exact original fragment sequence.
        sequence: u32,
    },
    /// This sequence was consumed earlier; no old bytes are reintroduced.
    Old {
        /// Exact original fragment sequence.
        sequence: u32,
    },
}

/// Receive update with an exclusive window borrow. Dropping cancels without mutation.
/// Validate the packet's outgoing ACK transaction separately before committing this.
#[must_use = "commit after whole-packet validation; dropping cancels the incoming update"]
pub struct PreparedFragment<'window, 'packet, 'buffers> {
    owner: &'window mut ReceiveWindow<'buffers>,
    payload: &'packet [u8],
    index: usize,
    summary: ReceiveDisposition,
}

impl<'buffers> ReceiveWindow<'buffers> {
    /// Preflights receive data only, without applying any ACK, flag or sender hint.
    /// The caller must validate those fields and the complete packet before commit.
    ///
    /// # Errors
    /// Rejects future out-of-window data and sequences whose consumption would
    /// overflow the native signed cumulative ACK domain. No state or bytes change.
    pub fn prepare<'window, 'packet>(
        &'window mut self,
        packet: ReliablePacket<'packet>,
    ) -> Result<PreparedFragment<'window, 'packet, 'buffers>, ReceiveError> {
        let mut index = 0;
        let summary = match packet.header().sequence {
            None => ReceiveDisposition::AckOnly,
            Some(sequence) => {
                if sequence == 0x7fff_ffff {
                    return Err(ReceiveError::SequenceExhausted);
                }
                if sequence < self.base {
                    ReceiveDisposition::Old { sequence }
                } else {
                    let distance = sequence - self.base;
                    if distance >= u32::from(self.span) {
                        return Err(ReceiveError::OutOfWindow);
                    }
                    let distance =
                        usize::try_from(distance).map_err(|_| ReceiveError::OutOfWindow)?;
                    index = (self.head + distance) % self.capacity();
                    if self.slots[index].sequence == Some(sequence) {
                        ReceiveDisposition::Duplicate { sequence }
                    } else {
                        ReceiveDisposition::Stored {
                            sequence,
                            bytes: packet.payload().len(),
                        }
                    }
                }
            }
        };
        Ok(PreparedFragment {
            owner: self,
            payload: packet.payload(),
            index,
            summary,
        })
    }
}

impl PreparedFragment<'_, '_, '_> {
    /// Returns metadata only, without changing retained state or copying payload bytes.
    #[must_use]
    pub fn summary(&self) -> ReceiveDisposition {
        self.summary
    }
    /// Applies the preflighted fragment infallibly under its exclusive storage borrow.
    /// Outgoing acknowledgement validation and send/timer policy remain the host's job.
    #[must_use]
    pub fn commit(self) -> ReceiveDisposition {
        if let ReceiveDisposition::Stored { sequence, bytes } = self.summary {
            let start = self.index * MAX_RELIABLE_PAYLOAD_BYTES;
            self.owner.bytes[start..start + bytes].copy_from_slice(self.payload);
            self.owner.slots[self.index] = ReceiveSlot {
                sequence: Some(sequence),
                length: bytes,
            };
            self.owner.occupied += 1;
            self.owner.buffered += bytes;
        }
        self.summary
    }
}

impl fmt::Debug for PreparedFragment<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedFragment")
            .field("summary", &self.summary)
            .finish_non_exhaustive()
    }
}
