// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Instant;

use cod4x_protocol::{MAX_PACKET_BYTES, Packet};
use cod4xr_client_core::BaselineProtocol;
use cod4xr_reliable::{
    MAX_RELIABLE_BODY_BYTES, ReliablePacket,
    receive::ReceiveAcknowledgement,
    send::{PreparedTransmission, SendWindow, SentFragment},
};

use crate::{ClientSender, NetworkError, datagram};

/// One auxiliary datagram accepted by the OS, without peer acknowledgement authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SentReliableDatagram {
    /// Actual data-send admission, absent for an ACK-only datagram.
    pub fragment: Option<SentFragment>,
    /// Complete datagram size, including the shared marker/qport and reliable headers.
    pub bytes: usize,
}

impl ClientSender<'_> {
    /// Sends at most one original reliable fragment on the acknowledged socket.
    /// Only complete successful OS I/O marks it transmitted. Errors drop preparation
    /// unmarked, preserving the original fragment for an explicitly paced retry.
    /// Baseline flags are zero; the original profile must be reviewed protocol 21.
    ///
    /// # Errors
    /// Rejects unsupported original profiles and invalid receive ACK metadata before I/O.
    /// Propagates shared-envelope, reliable-codec, absolute-deadline and socket errors.
    pub fn send_reliable_fragment(
        &mut self,
        transmission: PreparedTransmission<'_, '_>,
        receive: ReceiveAcknowledgement,
        deadline: Instant,
    ) -> Result<SentReliableDatagram, NetworkError> {
        self.check_reliable_profile()?;
        let packet = transmission
            .packet(receive.cumulative, receive.selective, 0)
            .map_err(NetworkError::Reliable)?;
        let bytes = self.send_reliable_packet(packet, deadline)?;
        Ok(SentReliableDatagram {
            fragment: Some(transmission.record_sent()),
            bytes,
        })
    }

    /// Sends at most one ACK-only datagram using the host's current receive snapshot.
    /// Uses the validated outgoing window's hint; no data admission or sequence reservation
    /// occurs. Scheduling, whole-packet receive validation and rate budgets remain the host's job.
    ///
    /// # Errors
    /// Rejects unsupported profiles/invalid ACK metadata before I/O, and propagates
    /// encoding, absolute-deadline and socket errors without changing either window.
    pub fn send_reliable_ack(
        &mut self,
        outgoing: &SendWindow<'_>,
        receive: ReceiveAcknowledgement,
        deadline: Instant,
    ) -> Result<SentReliableDatagram, NetworkError> {
        self.check_reliable_profile()?;
        let packet = outgoing
            .acknowledgement_packet(receive)
            .map_err(NetworkError::Reliable)?;
        let bytes = self.send_reliable_packet(packet, deadline)?;
        Ok(SentReliableDatagram {
            fragment: None,
            bytes,
        })
    }

    fn check_reliable_profile(&self) -> Result<(), NetworkError> {
        if BaselineProtocol::from_number(self.protocol) != Some(BaselineProtocol::Cod4x21) {
            return Err(NetworkError::AuxiliaryProfile);
        }
        Ok(())
    }

    fn send_reliable_packet(
        &mut self,
        packet: ReliablePacket<'_>,
        deadline: Instant,
    ) -> Result<usize, NetworkError> {
        let mut body = [0; MAX_RELIABLE_BODY_BYTES];
        let body_bytes = packet.encode(&mut body).map_err(NetworkError::Reliable)?;
        let mut wire = [0; MAX_PACKET_BYTES];
        let bytes = Packet::Reliable {
            qport: self.qport,
            body: &body[..body_bytes],
        }
        .encode(&mut wire)
        .map_err(NetworkError::Envelope)?;
        if let Err(error) = datagram::send(self.socket, &wire[..bytes], deadline) {
            self.usage.failed_datagrams = self.usage.failed_datagrams.saturating_add(1);
            self.usage.reliable_failed_datagrams =
                self.usage.reliable_failed_datagrams.saturating_add(1);
            return Err(error);
        }
        self.usage.datagrams_sent = self.usage.datagrams_sent.saturating_add(1);
        self.usage.reliable_datagrams_sent = self.usage.reliable_datagrams_sent.saturating_add(1);
        self.usage.bytes_sent = self
            .usage
            .bytes_sent
            .saturating_add(u64::try_from(bytes).unwrap_or(u64::MAX));
        Ok(bytes)
    }
}
