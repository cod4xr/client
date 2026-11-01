// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, net::UdpSocket, time::Instant};

use cod4x_protocol::MAX_PACKET_BYTES;
use cod4x_transport::{SendChannel, Transmission};
use cod4xr_client_core::control::ControlMessagePlan;

use crate::{NetworkError, SendUsage, datagram};

/// Borrowed sender; the separate receive half can be serviced between fragments.
pub struct ClientSender<'channel> {
    pub(crate) socket: &'channel UdpSocket,
    pub(crate) state: &'channel mut SendChannel,
    pub(crate) usage: &'channel mut SendUsage,
    pub(crate) protocol: i32,
    pub(crate) challenge: i32,
    pub(crate) qport: u16,
}

impl ClientSender<'_> {
    /// Reserves a sequence for an owned control frame from this protocol/cookie context.
    /// The caller must also supply the current validated game/header state when building
    /// the plan. Mark its included command prefix transmitted only after complete I/O.
    ///
    /// # Errors
    /// Context mismatch rejects before reserving a channel sequence or sending bytes.
    /// Shared message size/sequence failures are also propagated.
    pub fn begin_control<'sender, 'message>(
        &'sender mut self,
        plan: &'message ControlMessagePlan,
    ) -> Result<OutboundMessage<'sender, 'message>, NetworkError> {
        if plan.requested_protocol() != self.protocol || plan.server_challenge() != self.challenge {
            return Err(NetworkError::ControlContext);
        }
        self.begin(plan.as_bytes())
    }

    /// Reserves one sequence and borrows an already encoded message body.
    /// Dropping this transmission cancels it without reusing its sequence.
    ///
    /// # Errors
    /// Rejects oversized bodies and exhausted/reserved sequence ranges before reservation.
    pub fn begin<'sender, 'message>(
        &'sender mut self,
        body: &'message [u8],
    ) -> Result<OutboundMessage<'sender, 'message>, NetworkError> {
        let sequence = self.state.next_sequence();
        let transmission = self.state.begin(body).map_err(NetworkError::Channel)?;
        self.usage.messages_reserved = self.usage.messages_reserved.saturating_add(1);
        Ok(OutboundMessage {
            socket: self.socket,
            transmission,
            usage: self.usage,
            sequence,
            failed: false,
            packet: [0; MAX_PACKET_BYTES],
        })
    }

    /// Returns lifetime reservation/I/O counts, without bodies or qport.
    #[must_use]
    pub fn usage(&self) -> SendUsage {
        *self.usage
    }
    /// Returns the next sequence that a successful `begin` will reserve.
    #[must_use]
    pub fn next_sequence(&self) -> u32 {
        self.state.next_sequence()
    }
}

impl fmt::Debug for ClientSender<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientSender")
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}

/// Metadata for one complete datagram accepted by the OS; not peer acknowledgement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SentDatagram {
    /// The sequence reserved for this entire logical message.
    pub sequence: u32,
    /// Complete datagram bytes, including envelope headers.
    pub bytes: usize,
}

/// One borrowed ordinary transmission with at most one UDP send per operation.
pub struct OutboundMessage<'sender, 'message> {
    socket: &'sender UdpSocket,
    transmission: Transmission<'sender, 'message>,
    usage: &'sender mut SendUsage,
    sequence: u32,
    failed: bool,
    packet: [u8; MAX_PACKET_BYTES],
}

impl OutboundMessage<'_, '_> {
    /// Sends at most one fragment before the supplied absolute deadline.
    /// Returns `None` after the final datagram; the host controls pacing between calls.
    /// Exact multiples of 1300 include a zero-length terminating fragment.
    ///
    /// # Errors
    /// Propagates codec/I/O errors. The first failure terminates this transmission;
    /// retry by reserving a new message sequence, never by skipping a failed fragment.
    pub fn send_next(&mut self, deadline: Instant) -> Result<Option<SentDatagram>, NetworkError> {
        if self.failed {
            return Err(NetworkError::TransmissionFailed);
        }
        let length = match self.transmission.next_packet(&mut self.packet) {
            Ok(Some(length)) => length,
            Ok(None) => return Ok(None),
            Err(error) => {
                self.failed = true;
                return Err(NetworkError::Channel(error));
            }
        };
        if let Err(error) = datagram::send(self.socket, &self.packet[..length], deadline) {
            self.failed = true;
            self.usage.failed_datagrams = self.usage.failed_datagrams.saturating_add(1);
            return Err(error);
        }
        self.usage.datagrams_sent = self.usage.datagrams_sent.saturating_add(1);
        self.usage.bytes_sent = self
            .usage
            .bytes_sent
            .saturating_add(u64::try_from(length).unwrap_or(u64::MAX));
        Ok(Some(SentDatagram {
            sequence: self.sequence,
            bytes: length,
        }))
    }
}

impl fmt::Debug for OutboundMessage<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OutboundMessage")
            .field("sequence", &self.sequence)
            .field("failed", &self.failed)
            .finish_non_exhaustive()
    }
}
